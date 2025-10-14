use crate::error::ApiError;
use crate::generate_username::generate_random_username;
use crate::model::{AppState, CreateUserRequest, NewUser, User, UserResponse, UserStatus};
use crate::schema::users;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/create_user",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created successfully or already exists", body = UserResponse),
        (status = 400, description = "Invalid input", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    tag = "Users"
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // Validate wallet_address (addresses starts with 0x)
    if !req.wallet_address.starts_with("0x") {
        return Err(ApiError::Validation(
            "Invalid wallet address: must start with 0x".to_string(),
        ));
    }

    // Get database connection
    let conn = &mut state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;
    // Check if user exists
    let existing_user = users::table
        .filter(users::address.eq(&req.wallet_address))
        .select(User::as_select())
        .first(conn)
        .optional()?;

    if let Some(user) = existing_user {
        return Ok((
            StatusCode::OK,
            Json(UserResponse {
                address: user.address,
                username: user.username,
                position: user.position.unwrap_or(0),
                games_played: user.games_played.unwrap_or(0),
                is_registered: user.is_registered,
                highest_score: user.highest_score.unwrap_or(0),
                updated: user.updated,
                registered_at: user.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                status: Some(UserStatus::Existing),
            }),
        ));
    }

    let new_user = NewUser {
        address: req.wallet_address,
        username: generate_random_username(),
        is_registered: true,
        highest_score: Some(0),
        updated: false,
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(conn)?;

    Ok((
        StatusCode::OK,
        Json(UserResponse {
            address: user.address,
            username: user.username,
            position: user.position.unwrap_or(0),
            games_played: user.games_played.unwrap_or(0),
            is_registered: user.is_registered,
            highest_score: user.highest_score.unwrap_or(0),
            updated: user.updated,
            registered_at: user.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
            status: Some(UserStatus::New),
        }),
    ))
}