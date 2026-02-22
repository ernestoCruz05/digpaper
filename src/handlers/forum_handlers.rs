//! Forum handlers module
//!
//! HTTP handlers for forum-related endpoints including messages,
//! replies, voice messages, and task lists.

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::{
    CreateForumMessageRequest, CreateReplyRequest, ForumMessageResponse, TaskItemResponse,
    ToggleTaskItemRequest,
};
use crate::services::{ForumService, ProjectService, PushService};

use std::path::Path as StdPath;
use tokio::io::AsyncWriteExt;

/// Upload directory for audio files
const UPLOADS_DIR: &str = "./uploads";

/// POST /projects/:id/forum - Create a forum message
///
/// Supports JSON body for TEXT and TASK_LIST messages.
/// For VOICE messages, use the multipart endpoint instead.
pub async fn create_forum_message(
    State(pool): State<DbPool>,
    Path(project_id): Path<String>,
    Json(payload): Json<CreateForumMessageRequest>,
) -> AppResult<(StatusCode, Json<ForumMessageResponse>)> {
    let author = payload.author_name.as_deref().unwrap_or("Anónimo");

    let msg = match payload.message_type.as_str() {
        "TEXT" => {
            let content = payload.content.as_deref().unwrap_or("");
            ForumService::create_text_message(&pool, &project_id, content, author).await?
        }
        "TASK_LIST" => {
            let items = payload.items.as_deref().unwrap_or(&[]);
            ForumService::create_task_list(
                &pool,
                &project_id,
                payload.content.as_deref(),
                items,
                author,
            )
            .await?
        }
        _ => {
            return Err(crate::error::AppError::BadRequest(
                "Invalid message_type. Use TEXT or TASK_LIST.".to_string(),
            ))
        }
    };

    let response = ForumService::build_response(&pool, msg).await?;

    // Send push notifications in background
    let pool2 = pool.clone();
    let pid = project_id.clone();
    let author_owned = author.to_string();
    let content_owned = payload.content.unwrap_or_default();
    tokio::spawn(async move {
        let pname = ProjectService::get_by_id(&pool2, &pid)
            .await
            .map(|p| p.name)
            .unwrap_or_default();
        PushService::notify_new_message(&pool2, &pname, &author_owned, &content_owned).await;
    });

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /projects/:id/forum/voice - Create a voice message (multipart)
pub async fn create_voice_message(
    State(pool): State<DbPool>,
    Path(project_id): Path<String>,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<ForumMessageResponse>)> {
    let mut audio_path: Option<String> = None;
    let mut author_name = "Anónimo".to_string();
    let mut text_content: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| crate::error::AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "author_name" {
            if let Ok(text) = field.text().await {
                if !text.is_empty() {
                    author_name = text;
                }
            }
            continue;
        }

        if name == "content" {
            if let Ok(text) = field.text().await {
                if !text.is_empty() {
                    text_content = Some(text);
                }
            }
            continue;
        }

        if name == "audio" {
            let original_name = field.file_name().unwrap_or("voice.webm").to_string();
            let ext = StdPath::new(&original_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("webm");

            let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S");
            let short_id = &uuid::Uuid::new_v4().to_string()[..4];
            let filename = format!("voice_{}_{}.{}", timestamp, short_id, ext);

            let data = field
                .bytes()
                .await
                .map_err(|e| crate::error::AppError::Internal(format!("Read error: {}", e)))?;

            let file_path = format!("{}/{}", UPLOADS_DIR, filename);
            let mut file = tokio::fs::File::create(&file_path)
                .await
                .map_err(|e| crate::error::AppError::Internal(format!("File error: {}", e)))?;
            file.write_all(&data)
                .await
                .map_err(|e| crate::error::AppError::Internal(format!("Write error: {}", e)))?;

            audio_path = Some(filename);
        }
    }

    let path = audio_path
        .ok_or_else(|| crate::error::AppError::BadRequest("No audio file provided".to_string()))?;

    let msg =
        ForumService::create_voice_message(&pool, &project_id, &path, &author_name, text_content)
            .await?;
    let response = ForumService::build_response(&pool, msg).await?;

    // Push notification for voice message
    let pool2 = pool.clone();
    let pid = project_id.clone();
    let author_owned = author_name.clone();
    tokio::spawn(async move {
        let pname = ProjectService::get_by_id(&pool2, &pid)
            .await
            .map(|p| p.name)
            .unwrap_or_default();
        PushService::notify_new_message(&pool2, &pname, &author_owned, "🎙 Mensagem de voz").await;
    });

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /projects/:id/forum - List forum messages for a project
pub async fn list_forum_messages(
    State(pool): State<DbPool>,
    Path(project_id): Path<String>,
) -> AppResult<Json<Vec<ForumMessageResponse>>> {
    let messages = ForumService::list_messages(&pool, &project_id).await?;

    let mut responses = Vec::new();
    for msg in messages {
        let resp = ForumService::build_response(&pool, msg).await?;
        responses.push(resp);
    }

    Ok(Json(responses))
}

/// GET /forum/:msg_id/replies - List replies for a message
pub async fn list_replies(
    State(pool): State<DbPool>,
    Path(msg_id): Path<String>,
) -> AppResult<Json<Vec<ForumMessageResponse>>> {
    let replies = ForumService::list_replies(&pool, &msg_id).await?;

    let mut responses = Vec::new();
    for msg in replies {
        let resp = ForumService::build_response(&pool, msg).await?;
        responses.push(resp);
    }

    Ok(Json(responses))
}

/// POST /forum/:msg_id/reply - Create a reply to a message
pub async fn create_reply(
    State(pool): State<DbPool>,
    Path(msg_id): Path<String>,
    Json(payload): Json<CreateReplyRequest>,
) -> AppResult<(StatusCode, Json<ForumMessageResponse>)> {
    let author = payload.author_name.as_deref().unwrap_or("Anónimo");

    let msg = ForumService::create_reply(&pool, &msg_id, &payload.content, author).await?;
    let response = ForumService::build_response(&pool, msg).await?;

    // Push notification for reply
    let pool2 = pool.clone();
    let author_owned = author.to_string();
    let content_owned = payload.content.clone();
    tokio::spawn(async move {
        PushService::notify_new_message(&pool2, "Resposta", &author_owned, &content_owned).await;
    });

    Ok((StatusCode::CREATED, Json(response)))
}

/// PATCH /tasks/:item_id/toggle - Toggle a task item's completion
pub async fn toggle_task_item(
    State(pool): State<DbPool>,
    Path(item_id): Path<String>,
    Json(payload): Json<ToggleTaskItemRequest>,
) -> AppResult<Json<TaskItemResponse>> {
    let item =
        ForumService::toggle_task_item(&pool, &item_id, payload.completed_by.as_deref()).await?;
    Ok(Json(item.into()))
}
