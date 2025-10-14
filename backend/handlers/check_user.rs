// use crate::model::{AppState, User, UserResponse, UserStatus};
// use crate::schema::users;
// use axum::extract::State;
// use axum::http::StatusCode;
// use axum::response::IntoResponse;
// use axum::Json;
// use diesel::associations::HasTable;
// use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
// use serde::{Deserialize, Serialize};
// use std::sync::Arc;
// use utoipa::ToSchema;
//
// #[derive(Serialize, Deserialize, ToSchema)]
// pub struct CheckUserRequest {
//     pub wallet_address: String,
//     pub username: String,
// }
//
// #[utoipa::path(
//     post,
//     path = "/check_user",
//     request_body = CheckUserRequest,
//     responses(
//         (status = 200, description = "User found and username is available", body = UserResponse),
//         (status = 400, description = "Invalid input or username already taken", body = String),
//         (status = 404, description = "User not found", body = String),
//         (status = 500, description = "Internal server error", body = String)
//     ),
//     tag = "Users"
// )]
// pub async fn check_user(
//     State(state): State<Arc<AppState>>,
//     Json(req): Json<CheckUserRequest>,
// ) -> impl IntoResponse {
//     let conn = &mut state.db.get().expect("Couldn't get db connection from pool");
//
//     // Validate wallet address
//     if !req.wallet_address.starts_with("0x") {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Invalid Starknet address"})),
//         )
//             .into_response();
//     }
//
//     // Validate username
//     if req.username.is_empty() || req.username.len() > 50 {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Username must be between 1 and 50 characters"})),
//         )
//             .into_response();
//     }
//
//     if !req.username.chars().all(|c| c.is_alphanumeric() || c == '_') {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Username must contain only alphanumeric characters or underscores"})),
//         )
//             .into_response();
//     }
//
//     // Check if username is already taken
//     let username_exists = match users::table
//         .filter(users::username.eq(&req.username))
//         .select(users::id)
//         .first::<i32>(conn)
//         .optional()
//     {
//         Ok(opt) => opt,
//         Err(e) => {
//             return (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(serde_json::json!({"error": format!("Database error: {}", e)})),
//             )
//                 .into_response();
//         }
//     };
//
//     if username_exists.is_some() {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Username already taken"})),
//         )
//             .into_response();
//     }
//
//     // Check if user exists by wallet address
//     let user = match users::table
//         .filter(users::address.eq(&req.wallet_address))
//         .select(User::as_select())
//         .first::<User>(conn)
//         .optional()
//     {
//         Ok(opt) => opt,
//         Err(e) => {
//             return (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(serde_json::json!({"error": format!("Database error: {}", e)})),
//             )
//                 .into_response();
//         }
//     };
//
//     match user {
//         Some(user) => (
//             StatusCode::OK,
//             Json(UserResponse {
//                 id: user.id,
//                 address: user.address,
//                 username: user.username,
//                 position: user.position.unwrap_or(0),
//                 games_played: user.games_played.unwrap(),
//                 is_registered: user.is_registered,
//                 highest_score: user.highest_score.unwrap_or(0),
//                 updated: user.updated,
//                 registered_at: user.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
//                 status: Some(UserStatus::Existing),
//             }),
//         )
//             .into_response(),
//         None => (
//             StatusCode::NOT_FOUND,
//             Json(serde_json::json!({"error": "User not found"})),
//         )
//             .into_response(),
//     }
// }

use crate::error::ApiError;
use crate::model::{AppState, User, UserResponse, UserStatus};
use crate::schema::users;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use diesel::sql_types::Uuid;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CheckUserRequest {
    pub wallet_address: String,
    pub username: String,
}

#[utoipa::path(
    post,
    path = "/check_user",
    request_body = CheckUserRequest,
    responses(
        (status = 200, description = "User found and username is available", body = UserResponse),
        (status = 400, description = "Invalid input or username already taken", body = serde_json::Value),
        (status = 404, description = "User not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    tag = "Users"
)]
pub async fn check_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckUserRequest>,
) -> impl IntoResponse {
    // Validate wallet_address (addresses starts with 0x)
    if !req.wallet_address.starts_with("0x") {
        return Err(ApiError::Validation(
            "Invalid wallet address: must start with 0x".to_string(),
        ));
    }

    // Validate username
    if req.username.is_empty() || req.username.len() > 50 {
        return Err(ApiError::Validation(
            "Username must be between 1 and 50 characters".to_string(),
        ));
    }
    if !req.username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(ApiError::Validation(
            "Username must contain only alphanumeric characters or underscores".to_string(),
        ));
    }

    // Get database connection
    let conn = &mut state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Check if username is already taken
    let username_exists = users::table
        .filter(users::username.eq(&req.username))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)
        .map_err(ApiError::Database)
        .unwrap();

    if username_exists {
        return Err(ApiError::Validation("Username already taken".to_string()));
    }

    // Check if user exists by wallet address
    let user = users::table
        .filter(users::address.eq(&req.wallet_address))
        .select(User::as_select())
        .first(conn)
        .optional()?;

    match user {
        Some(user) => Ok((
            StatusCode::OK,
            Json(UserResponse {
                address: user.address,
                username: user.username,
                position: user.position.unwrap_or(0),
                games_played: user.games_played.unwrap_or(0), // Fixed potential unwrap issue
                is_registered: user.is_registered,
                highest_score: user.highest_score.unwrap_or(0),
                updated: user.updated,
                registered_at: user.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                status: Some(UserStatus::Existing),
            }),
        )),
        None => Err(ApiError::Database(diesel::result::Error::NotFound)),
    }
}