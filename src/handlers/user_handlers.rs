use crate::{
    db::DbPool,
    error::AppResult,
    models::{UpdateProfilePhotoRequest, UserProfile},
    services::user_service::UserService,
};
use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};

/// GET /api/profiles - List all user profiles
async fn list_profiles(State(pool): State<DbPool>) -> AppResult<Json<Vec<UserProfile>>> {
    let profiles = UserService::list_profiles(&pool).await?;
    Ok(Json(profiles))
}

/// PUT /api/profiles/:name/photo - Update a user's profile photo
async fn update_profile_photo(
    State(pool): State<DbPool>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateProfilePhotoRequest>,
) -> AppResult<Json<UserProfile>> {
    let profile = UserService::update_profile_photo(&pool, &name, &payload.photo_url).await?;
    Ok(Json(profile))
}

/// Router configuration for user profile endpoints
pub fn router() -> Router<DbPool> {
    Router::new()
        .route("/", get(list_profiles))
        .route("/:name/photo", put(update_profile_photo))
}
