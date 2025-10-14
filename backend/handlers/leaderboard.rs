use crate::error::ApiError;
use crate::model::{AppState, User, UserResponse};
use crate::schema::users;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;
use utoipa::ToSchema;

#[utoipa::path(
    get,
    path = "/leaderboard",
    responses(
        (status = 200, description = "Top 20 users by highest score", body = [UserResponse]),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    tag = "Users"
)]
pub async fn leaderboard(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<UserResponse>>), ApiError> {
    // Get database connection
    let conn = &mut state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Query top 20 users by highest_score
    let top_users = users::table
        .order(users::highest_score.desc())
        .limit(20)
        .select(User::as_select())
        .load::<User>(conn)?;

    // Map to UserResponse
    let response = top_users
        .into_iter()
        .map(|usr| UserResponse {
            address: usr.address,
            username: usr.username,
            position: usr.position.unwrap_or(0),
            games_played: usr.games_played.unwrap_or(0),
            is_registered: usr.is_registered,
            highest_score: usr.highest_score.unwrap_or(0),
            updated: usr.updated,
            registered_at: usr.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
            status: None,
        })
        .collect::<Vec<UserResponse>>();

    Ok((StatusCode::OK, Json(response)))
}