//! Push notification handlers
//!
//! Endpoints for managing web push subscriptions and retrieving VAPID public key.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::db::DbPool;
use crate::error::AppResult;
use crate::services::PushService;

#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub author_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnsubscribeRequest {
    pub endpoint: String,
}

/// GET /push/vapid-key - Get the VAPID public key for subscribing
pub async fn get_vapid_key(State(pool): State<DbPool>) -> AppResult<Json<serde_json::Value>> {
    let key = PushService::get_vapid_public_key(&pool).await?;
    Ok(Json(serde_json::json!({ "publicKey": key })))
}

/// POST /push/subscribe - Register a push subscription
pub async fn push_subscribe(
    State(pool): State<DbPool>,
    Json(payload): Json<SubscribeRequest>,
) -> AppResult<StatusCode> {
    PushService::subscribe(
        &pool,
        payload.endpoint,
        payload.p256dh,
        payload.auth,
        payload.author_name,
    )
    .await?;
    Ok(StatusCode::CREATED)
}

/// POST /push/unsubscribe - Remove a push subscription
pub async fn push_unsubscribe(
    State(pool): State<DbPool>,
    Json(payload): Json<UnsubscribeRequest>,
) -> AppResult<StatusCode> {
    PushService::unsubscribe(&pool, &payload.endpoint).await?;
    Ok(StatusCode::NO_CONTENT)
}
