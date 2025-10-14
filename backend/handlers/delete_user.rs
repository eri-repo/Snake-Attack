// use crate::model::{AppState, User, UserResponse};
// use crate::schema::users;
// use axum::extract::State;
// use axum::http::StatusCode;
// use axum::response::IntoResponse;
// use axum::Json;
// use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
// use serde::{Deserialize, Serialize};
// use std::sync::Arc;
// use utoipa::ToSchema;
//
// #[derive(Serialize, Deserialize, ToSchema)]
// pub struct DeleteUserRequest {
//     pub wallet_address: String,
// }
//
// #[utoipa::path(
//     delete,
//     path = "/delete_user",
//     request_body = DeleteUserRequest,
//     responses(
//         (status = 200, description = "User deleted successfully", body = UserResponse),
//         (status = 400, description = "Invalid Starknet address", body = String),
//         (status = 404, description = "User not found", body = String),
//         (status = 500, description = "Internal server error", body = String)
//     ),
//     tag = "Users"
// )]
// pub async fn delete_user(
//     State(state): State<Arc<AppState>>,
//     Json(req): Json<DeleteUserRequest>,
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
//     // Check if user exists
//     let existing_user = match users::table
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
//     if let Some(user) = existing_user {
//         // Delete the user
//         match diesel::delete(users::table.filter(users::address.eq(&req.wallet_address)))
//             .execute(conn)
//         {
//             Ok(_) => (
//                 StatusCode::OK,
//                 Json(UserResponse {
//                     id: user.id,
//                     address: user.address,
//                     username: user.username,
//                     position: user.position.unwrap_or(0),
//                     games_played: user.games_played.unwrap(),
//                     is_registered: user.is_registered,
//                     highest_score: user.highest_score.unwrap_or(0),
//                     updated: user.updated,
//                     registered_at: user.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
//                     status: None,
//                 }),
//             )
//                 .into_response(),
//             Err(e) => (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(serde_json::json!({"error": format!("Error deleting user: {}", e)})),
//             )
//                 .into_response(),
//         }
//     } else {
//         (
//             StatusCode::NOT_FOUND,
//             Json(serde_json::json!({"error": "User not found"})),
//         )
//             .into_response()
//     }
// }

use crate::error::ApiError;
use crate::model::{AppState, User, UserResponse};
use crate::schema::users;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DeleteUserRequest {
    pub wallet_address: String,
}

#[utoipa::path(
    delete,
    path = "/delete_user",
    request_body = DeleteUserRequest,
    responses(
        (status = 200, description = "User deleted successfully", body = UserResponse),
        (status = 400, description = "Invalid input", body = serde_json::Value),
        (status = 404, description = "User not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    tag = "Users"
)]
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteUserRequest>,
) -> impl IntoResponse {
    // Validate wallet_address (Starknet addresses are 0x + 64 hex chars)
    let re = Regex::new(r"^0x[0-9a-fA-F]{64}$").expect("Invalid regex");
    if !re.is_match(&req.wallet_address) {
        return Err(ApiError::Validation(
            "Invalid Starknet address: must be 0x followed by 64 hexadecimal characters".to_string(),
        ));
    }

    // Get database connection
    let conn = &mut state
        .db
        .get()
        .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

    // Check if user exists
    let user = users::table
        .filter(users::address.eq(&req.wallet_address))
        .select(User::as_select())
        .first(conn)
        .optional()?;

    match user {
        Some(user) => {
            // Delete the user
            diesel::delete(users::table.filter(users::address.eq(&req.wallet_address)))
                .execute(conn)?;

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
                    status: None,
                }),
            ))
        }
        None => Err(ApiError::Database(diesel::result::Error::NotFound)),
    }
}