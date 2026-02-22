use crate::error::AppResult;
use crate::models::UserProfile;
use sqlx::SqlitePool;

pub struct UserService;

impl UserService {
    /// Retrieve all user profiles
    pub async fn list_profiles(pool: &SqlitePool) -> AppResult<Vec<UserProfile>> {
        let profiles = sqlx::query_as::<_, UserProfile>(
            "SELECT name, photo_url, updated_at FROM user_profiles ORDER BY name ASC",
        )
        .fetch_all(pool)
        .await?;

        Ok(profiles)
    }

    /// Retrieve a specific user profile by name
    pub async fn get_profile(pool: &SqlitePool, name: &str) -> AppResult<Option<UserProfile>> {
        let profile = sqlx::query_as::<_, UserProfile>(
            "SELECT name, photo_url, updated_at FROM user_profiles WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(profile)
    }

    /// Update a user's profile photo. Creates the profile if it doesn't exist.
    pub async fn update_profile_photo(
        pool: &SqlitePool,
        name: &str,
        photo_url: &str,
    ) -> AppResult<UserProfile> {
        // Upsert the profile
        sqlx::query(
            r#"
            INSERT INTO user_profiles (name, photo_url, updated_at)
            VALUES (?, ?, datetime('now'))
            ON CONFLICT(name) DO UPDATE SET 
                photo_url = excluded.photo_url,
                updated_at = datetime('now')
            "#,
        )
        .bind(name)
        .bind(photo_url)
        .execute(pool)
        .await?;

        // Return the updated profile
        let profile = Self::get_profile(pool, name)
            .await?
            .ok_or_else(|| crate::error::AppError::Database(sqlx::Error::RowNotFound))?;

        Ok(profile)
    }
}
