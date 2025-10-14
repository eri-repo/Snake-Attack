use crate::error::ApiError;
use crate::model::{AppState, User, UserResponse};
use crate::schema::users;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateScoreRequest {
    pub wallet_address: String,
    pub score: i64,
    pub games_played: i64,
}

#[utoipa::path(
    put,
    path = "/update_score",
    request_body = UpdateScoreRequest,
    responses(
        (status = 200, description = "Score and games played updated successfully", body = UserResponse),
        (status = 400, description = "Invalid input", body = serde_json::Value),
        (status = 404, description = "User not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    tag = "Users"
)]
pub async fn update_score(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateScoreRequest>,
) -> impl IntoResponse {
    // Validate wallet_address (addresses starts with 0x)
    if !req.wallet_address.starts_with("0x") {
        return Err(ApiError::Validation(
            "Invalid wallet address: must start with 0x".to_string(),
        ));
    }

    // Validate score and games_played
    if req.score < 0 {
        return Err(ApiError::Validation(
            "Score must be non-negative".to_string(),
        ));
    }
    if req.games_played < 0 {
        return Err(ApiError::Validation(
            "Games played must be non-negative".to_string(),
        ));
    }

    // Get database connection
    let conn = &mut state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Check if user exists
    let user = users::table
        .filter(users::address.eq(&req.wallet_address))
        .select(User::as_select())
        .first(conn)
        .optional()?;

    match user {
        Some(user) => {
            // Update highest_score if new score is higher, and set games_played
            let new_highest_score = user.highest_score.unwrap_or(0) + req.score;
            let new_games_played = user.games_played.unwrap_or(0) + req.games_played;

            let updated_user =
                diesel::update(users::table.filter(users::address.eq(&req.wallet_address)))
                    .set((
                        users::highest_score.eq(new_highest_score),
                        users::games_played.eq(req.games_played),
                    ))
                    .get_result::<User>(conn)?;

            Ok((
                StatusCode::OK,
                Json(UserResponse {
                    address: updated_user.address,
                    username: updated_user.username,
                    position: updated_user.position.unwrap_or(0),
                    games_played: updated_user.games_played.unwrap_or(0),
                    is_registered: updated_user.is_registered,
                    highest_score: updated_user.highest_score.unwrap_or(0),
                    updated: updated_user.updated,
                    registered_at: updated_user
                        .registered_at
                        .format("%Y-%m-%dT%H:%M:%S")
                        .to_string(),
                    status: None,
                }),
            ))
        }
        None => Err(ApiError::Database(diesel::result::Error::NotFound)),
    }
}
