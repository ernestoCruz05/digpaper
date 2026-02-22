//! Forum service module
//!
//! Business logic for forum messages, replies, and task items.
//! Handles both per-Obra forums and the global Geral forum.

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{
    Document, DocumentResponse, ForumMessage, ForumMessageResponse, TaskItem, TaskItemResponse,
};

/// Forum service with static methods for forum operations
pub struct ForumService;

impl ForumService {
    /// Create a text message in a forum
    pub async fn create_text_message(
        pool: &DbPool,
        project_id: &str,
        content: &str,
        author_name: &str,
    ) -> AppResult<ForumMessage> {
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO forum_messages (id, project_id, message_type, content, author_name) VALUES (?, ?, 'TEXT', ?, ?)",
        )
        .bind(&id)
        .bind(project_id)
        .bind(content)
        .bind(author_name)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create message: {}", e)))?;

        Self::get_by_id(pool, &id).await
    }

    /// Create a photo message (auto-posted when a document is uploaded to an Obra)
    pub async fn create_photo_message(
        pool: &DbPool,
        project_id: &str,
        document_id: &str,
        author_name: &str,
    ) -> AppResult<ForumMessage> {
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO forum_messages (id, project_id, message_type, document_id, author_name) VALUES (?, ?, 'PHOTO', ?, ?)",
        )
        .bind(&id)
        .bind(project_id)
        .bind(document_id)
        .bind(author_name)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create photo message: {}", e)))?;

        Self::get_by_id(pool, &id).await
    }

    pub async fn create_voice_message(
        pool: &DbPool,
        project_id: &str,
        audio_path: &str,
        author_name: &str,
        content: Option<String>,
    ) -> AppResult<ForumMessage> {
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO forum_messages (id, project_id, message_type, audio_path, author_name, content) VALUES (?, ?, 'VOICE', ?, ?, ?)",
        )
        .bind(&id)
        .bind(project_id)
        .bind(audio_path)
        .bind(author_name)
        .bind(content)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create voice message: {}", e)))?;

        Self::get_by_id(pool, &id).await
    }

    /// Create a task list message with items
    pub async fn create_task_list(
        pool: &DbPool,
        project_id: &str,
        content: Option<&str>,
        items: &[String],
        author_name: &str,
    ) -> AppResult<ForumMessage> {
        let msg_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO forum_messages (id, project_id, message_type, content, author_name) VALUES (?, ?, 'TASK_LIST', ?, ?)",
        )
        .bind(&msg_id)
        .bind(project_id)
        .bind(content)
        .bind(author_name)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create task list: {}", e)))?;

        // Insert each task item
        for item_text in items {
            let item_id = uuid::Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO task_items (id, message_id, text) VALUES (?, ?, ?)")
                .bind(&item_id)
                .bind(&msg_id)
                .bind(item_text)
                .execute(pool)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to create task item: {}", e)))?;
        }

        Self::get_by_id(pool, &msg_id).await
    }

    /// Create a reply to an existing message
    pub async fn create_reply(
        pool: &DbPool,
        parent_id: &str,
        content: &str,
        author_name: &str,
    ) -> AppResult<ForumMessage> {
        // Get parent to inherit project_id
        let parent = Self::get_by_id(pool, parent_id).await?;

        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO forum_messages (id, project_id, parent_id, message_type, content, author_name) VALUES (?, ?, ?, 'TEXT', ?, ?)",
        )
        .bind(&id)
        .bind(&parent.project_id)
        .bind(parent_id)
        .bind(content)
        .bind(author_name)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create reply: {}", e)))?;

        Self::get_by_id(pool, &id).await
    }

    /// List top-level messages for a project's forum (no replies)
    pub async fn list_messages(pool: &DbPool, project_id: &str) -> AppResult<Vec<ForumMessage>> {
        let messages: Vec<ForumMessage> = sqlx::query_as(
            "SELECT * FROM forum_messages WHERE project_id = ? AND parent_id IS NULL ORDER BY created_at ASC",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list messages: {}", e)))?;

        Ok(messages)
    }

    /// List replies for a specific message
    pub async fn list_replies(pool: &DbPool, message_id: &str) -> AppResult<Vec<ForumMessage>> {
        let replies: Vec<ForumMessage> = sqlx::query_as(
            "SELECT * FROM forum_messages WHERE parent_id = ? ORDER BY created_at ASC",
        )
        .bind(message_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list replies: {}", e)))?;

        Ok(replies)
    }

    /// Get reply count for a message
    pub async fn get_reply_count(pool: &DbPool, message_id: &str) -> AppResult<i32> {
        let count: (i32,) =
            sqlx::query_as("SELECT COUNT(*) FROM forum_messages WHERE parent_id = ?")
                .bind(message_id)
                .fetch_one(pool)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to count replies: {}", e)))?;

        Ok(count.0)
    }

    /// Get task items for a TASK_LIST message
    pub async fn get_task_items(pool: &DbPool, message_id: &str) -> AppResult<Vec<TaskItem>> {
        let items: Vec<TaskItem> =
            sqlx::query_as("SELECT * FROM task_items WHERE message_id = ? ORDER BY created_at ASC")
                .bind(message_id)
                .fetch_all(pool)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to get task items: {}", e)))?;

        Ok(items)
    }

    /// Toggle a task item's completion status
    pub async fn toggle_task_item(
        pool: &DbPool,
        item_id: &str,
        completed_by: Option<&str>,
    ) -> AppResult<TaskItem> {
        // Get current state
        let item: TaskItem = sqlx::query_as("SELECT * FROM task_items WHERE id = ?")
            .bind(item_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get task item: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Task item not found".to_string()))?;

        let new_completed = !item.completed;
        let completed_at = if new_completed {
            Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
        } else {
            None
        };
        let completed_by_val = if new_completed { completed_by } else { None };

        sqlx::query(
            "UPDATE task_items SET completed = ?, completed_by = ?, completed_at = ? WHERE id = ?",
        )
        .bind(new_completed)
        .bind(completed_by_val)
        .bind(&completed_at)
        .bind(item_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to toggle task item: {}", e)))?;

        let updated: TaskItem = sqlx::query_as("SELECT * FROM task_items WHERE id = ?")
            .bind(item_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get updated task item: {}", e)))?;

        Ok(updated)
    }

    /// Get a single message by ID
    pub async fn get_by_id(pool: &DbPool, id: &str) -> AppResult<ForumMessage> {
        let msg: ForumMessage = sqlx::query_as("SELECT * FROM forum_messages WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get message: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Message not found".to_string()))?;

        Ok(msg)
    }

    /// Build a full response for a message, including reply count, document info, and task items
    pub async fn build_response(
        pool: &DbPool,
        msg: ForumMessage,
    ) -> AppResult<ForumMessageResponse> {
        let reply_count = Self::get_reply_count(pool, &msg.id).await?;

        // Get document info for PHOTO messages
        let document = if msg.message_type == "PHOTO" {
            if let Some(ref doc_id) = msg.document_id {
                let doc: Option<Document> = sqlx::query_as("SELECT * FROM documents WHERE id = ?")
                    .bind(doc_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to get document: {}", e)))?;
                doc.map(DocumentResponse::from_document)
            } else {
                None
            }
        } else {
            None
        };

        // Get task items for TASK_LIST messages
        let items = if msg.message_type == "TASK_LIST" {
            let task_items = Self::get_task_items(pool, &msg.id).await?;
            Some(task_items.into_iter().map(TaskItemResponse::from).collect())
        } else {
            None
        };

        let audio_url = msg.audio_path.as_ref().map(|p| format!("/files/{}", p));

        Ok(ForumMessageResponse {
            id: msg.id,
            project_id: msg.project_id,
            parent_id: msg.parent_id,
            message_type: msg.message_type,
            content: msg.content,
            document_id: msg.document_id,
            audio_path: msg.audio_path,
            audio_url,
            author_name: msg.author_name,
            created_at: msg.created_at,
            reply_count,
            document,
            items,
        })
    }
}
