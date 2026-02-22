//! Push notification service
//!
//! Manages VAPID keys, push subscriptions, and sending notifications.
//! VAPID keys are auto-generated on first run and stored in the database.

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use std::process::Command;
use uuid::Uuid;
use web_push::*;

/// Push subscription stored in DB
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct PushSubscription {
    pub id: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub author_name: Option<String>,
    pub created_at: String,
}

pub struct PushService;

impl PushService {
    /// Initialize VAPID keys - generates new ones via openssl if they don't exist
    pub async fn init_vapid(pool: &DbPool) -> AppResult<()> {
        let existing: Option<(String,)> =
            sqlx::query_as("SELECT value FROM app_settings WHERE key = 'vapid_private_pem'")
                .fetch_optional(pool)
                .await?;

        if existing.is_some() {
            tracing::info!("VAPID keys already exist");
            return Ok(());
        }

        tracing::info!("Generating new VAPID keys via openssl...");

        // Generate EC private key
        let output = Command::new("openssl")
            .args(["ecparam", "-genkey", "-name", "prime256v1", "-noout"])
            .output()
            .map_err(|e| AppError::Internal(format!("openssl not found: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Internal("Failed to generate EC key".into()));
        }

        let private_key_der = output.stdout;

        // Convert to PEM format
        let pem_output = Command::new("openssl")
            .args(["ec", "-outform", "PEM"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.take().unwrap().write_all(&private_key_der)?;
                child.wait_with_output()
            })
            .map_err(|e| AppError::Internal(format!("Failed to convert key: {}", e)))?;

        let private_pem = String::from_utf8(pem_output.stdout)
            .map_err(|_| AppError::Internal("Invalid PEM output".into()))?;

        // Extract public key in DER format for the applicationServerKey
        let pub_output = Command::new("openssl")
            .args(["ec", "-pubout", "-outform", "DER"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.take().unwrap().write_all(&private_key_der)?;
                child.wait_with_output()
            })
            .map_err(|e| AppError::Internal(format!("Failed to extract public key: {}", e)))?;

        // Last 65 bytes of DER-encoded public key is the raw uncompressed point
        let pub_der = pub_output.stdout;
        let raw_public = if pub_der.len() >= 65 {
            &pub_der[pub_der.len() - 65..]
        } else {
            return Err(AppError::Internal("Public key too short".into()));
        };

        let public_b64 = URL_SAFE_NO_PAD.encode(raw_public);

        // Store keys in database
        sqlx::query(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('vapid_private_pem', ?)",
        )
        .bind(&private_pem)
        .execute(pool)
        .await?;

        sqlx::query(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('vapid_public_key', ?)",
        )
        .bind(&public_b64)
        .execute(pool)
        .await?;

        tracing::info!("VAPID keys generated. Public key: {}", public_b64);
        Ok(())
    }

    /// Get the VAPID public key (for frontend to subscribe)
    pub async fn get_vapid_public_key(pool: &DbPool) -> AppResult<String> {
        let row: (String,) =
            sqlx::query_as("SELECT value FROM app_settings WHERE key = 'vapid_public_key'")
                .fetch_one(pool)
                .await
                .map_err(|_| AppError::Internal("VAPID keys not initialized".into()))?;

        Ok(row.0)
    }

    /// Save a push subscription
    pub async fn subscribe(
        pool: &DbPool,
        endpoint: String,
        p256dh: String,
        auth: String,
        author_name: Option<String>,
    ) -> AppResult<()> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO push_subscriptions (id, endpoint, p256dh, auth, author_name)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&endpoint)
        .bind(&p256dh)
        .bind(&auth)
        .bind(&author_name)
        .execute(pool)
        .await?;

        tracing::info!("Push subscription saved for {:?}", author_name);
        Ok(())
    }

    /// Remove a push subscription
    pub async fn unsubscribe(pool: &DbPool, endpoint: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = ?")
            .bind(endpoint)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Send notification to all subscribers except the sender
    pub async fn notify_new_message(
        pool: &DbPool,
        project_name: &str,
        author_name: &str,
        content: &str,
    ) {
        // Get VAPID private PEM
        let private_pem: Option<(String,)> =
            sqlx::query_as("SELECT value FROM app_settings WHERE key = 'vapid_private_pem'")
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();

        let Some((ref pem_str,)) = private_pem else {
            tracing::debug!("No VAPID keys, skipping push");
            return;
        };

        // Get all subscriptions except sender's
        let subs: Vec<PushSubscription> = sqlx::query_as(
            "SELECT * FROM push_subscriptions WHERE author_name IS NULL OR author_name != ?",
        )
        .bind(author_name)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        if subs.is_empty() {
            return;
        }

        let payload = serde_json::json!({
            "title": format!("{} - {}", project_name, author_name),
            "body": if content.len() > 100 { format!("{}...", &content[..97]) } else { content.to_string() },
            "tag": project_name,
        });
        let payload_str = payload.to_string();

        for sub in subs {
            let sub_info = SubscriptionInfo::new(&sub.endpoint, &sub.p256dh, &sub.auth);

            let sig_builder = match VapidSignatureBuilder::from_pem(pem_str.as_bytes(), &sub_info) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("VAPID sig error: {:?}", e);
                    continue;
                }
            };

            let signature = match sig_builder.build() {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("VAPID build error: {:?}", e);
                    continue;
                }
            };

            let mut builder = WebPushMessageBuilder::new(&sub_info);
            let _ = builder.set_payload(ContentEncoding::Aes128Gcm, payload_str.as_bytes());
            builder.set_vapid_signature(signature);

            let message = match builder.build() {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Push message build error: {:?}", e);
                    continue;
                }
            };

            let client = match IsahcWebPushClient::new() {
                Ok(c) => c,
                Err(_) => continue,
            };

            match client.send(message).await {
                Ok(_) => {
                    tracing::debug!("Push sent to {:?}", sub.author_name);
                }
                Err(e) => {
                    let err_str = format!("{:?}", e);
                    if err_str.contains("EndpointNotValid")
                        || err_str.contains("EndpointNotFound")
                        || err_str.contains("410")
                    {
                        // Subscription expired
                        let _ = sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = ?")
                            .bind(&sub.endpoint)
                            .execute(pool)
                            .await;
                        tracing::info!("Removed expired subscription");
                    } else {
                        tracing::warn!("Push failed: {:?}", e);
                    }
                }
            }
        }
    }
}
