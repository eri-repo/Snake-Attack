// use crate::model::{AppState, UpdateUserRequest, User, UserResponse};
// use crate::schema::users;
// use axum::extract::State;
// use axum::http::StatusCode;
// use axum::response::IntoResponse;
// use axum::Json;
// use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
// use std::sync::Arc;
//
// #[utoipa::path(
//     put,
//     path = "/update_user",
//     request_body = UpdateUserRequest,
//     responses(
//         (status = 200, description = "Username updated successfully", body = UserResponse),
//         (status = 400, description = "Invalid input, update limit reached, or username already taken", body = String),
//         (status = 404, description = "User not found", body = String),
//         (status = 500, description = "Internal server error", body = String)
//     ),
//     tag = "Users"
// )]
// pub async fn update_user(
//     State(state): State<Arc<AppState>>,
//     Json(req): Json<UpdateUserRequest>,
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
//     if req.new_username.is_empty() || req.new_username.len() > 50 {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Username must be between 1 and 50 characters"})),
//         )
//             .into_response();
//     }
//
//     if !req.new_username.chars().all(|c| c.is_alphanumeric() || c == '_') {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Username must contain only alphanumeric characters or underscores"})),
//         )
//             .into_response();
//     }
//
//     // Check if user exists and get update status
//     let user = users::table
//         .filter(users::address.eq(&req.wallet_address))
//         .select((users::updated, users::id, users::username))
//         .first::<(bool, i32, String)>(conn)
//         .optional()
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(serde_json::json!({"error": format!("Database error: {}", e)})),
//             )
//                 .into_response()
//         })
//         .unwrap();
//
//     match user {
//         Some((true, _, _)) => {
//             return (
//                 StatusCode::BAD_REQUEST,
//                 Json(serde_json::json!({"error": "Username update limit reached: only one update allowed"})),
//             )
//                 .into_response();
//         }
//         Some((_, _id, current_username)) => {
//             // Check if new_username is already taken by another user
//             if current_username != req.new_username {
//                 let username_exists = match users::table
//                     .filter(users::username.eq(&req.new_username))
//                     // .filter(users::address.ne(&req.wallet_address))
//                     .select(users::id)
//                     .first::<i32>(conn)
//                     .optional()
//                 {
//                     Ok(opt) => opt,
//                     Err(e) => {
//                         return (
//                             StatusCode::INTERNAL_SERVER_ERROR,
//                             Json(serde_json::json!({"error": format!("Database error: {}", e)})),
//                         )
//                             .into_response();
//                     }
//                 };
//
//                 if username_exists.is_some() {
//                     return (
//                         StatusCode::BAD_REQUEST,
//                         Json(serde_json::json!({"error": "Username already taken"})),
//                     )
//                         .into_response();
//                 }
//             }
//         }
//         None => {
//             return (
//                 StatusCode::NOT_FOUND,
//                 Json(serde_json::json!({"error": "User not found"})),
//             )
//                 .into_response();
//         }
//     }
//
//     // Update the user
//     match diesel::update(users::table.filter(users::address.eq(&req.wallet_address)))
//         .set((
//             users::username.eq(&req.new_username),
//             users::updated.eq(true),
//         ))
//         .get_result::<User>(conn)
//     {
//         Ok(user) => (
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
//                 status: None,
//             }),
//         )
//             .into_response(),
//         Err(e) => (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(serde_json::json!({"error": format!("Error updating user: {}", e)})),
//         )
//             .into_response(),
//     }
// }

use crate::error::ApiError;
use crate::model::{AppState, User, UserResponse};
use crate::schema::users;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use diesel::sql_types::Uuid;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub wallet_address: String,
    pub new_username: String,
}

#[utoipa::path(
    put,
    path = "/update_user",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Username updated successfully", body = UserResponse),
        (status = 400, description = "Invalid input, update limit reached, or username already taken", body = serde_json::Value),
        (status = 404, description = "User not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    tag = "Users"
)]
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    // Validate wallet_address (addresses starts with 0x)
    if !req.wallet_address.starts_with("0x") {
        return Err(ApiError::Validation(
            "Invalid wallet address: must start with 0x".to_string(),
        ));
    }

    // Validate new_username
    let re_username = Regex::new(r"^[a-zA-Z0-9_]{1,50}$").expect("Invalid regex");
    if !re_username.is_match(&req.new_username) {
        return Err(ApiError::Validation(
            "Username must be 1-50 characters and contain only alphanumeric characters or underscores"
                .to_string(),
        ));
    }

    // Get database connection
    let conn = &mut state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Check if user exists and get update status
    let user = users::table
        .filter(users::address.eq(&req.wallet_address))
        .select((users::updated, users::id, users::username))
        .first::<(bool, i32, String)>(conn)
        .optional()?;

    match user {
        Some((true, _, _)) => Err(ApiError::Validation(
            "Username update limit reached: only one update allowed".to_string(),
        )),
        Some((_, id, current_username)) => {
            // Check if new_username is already taken by another user
            if current_username != req.new_username {
                // Check if username is already taken
                let username_exists = users::table
                    .filter(users::username.eq(&req.new_username))
                    .select(diesel::dsl::count_star())
                    .first::<i64>(conn)
                    .map(|count| count > 0)
                    .map_err(ApiError::Database)
                    .unwrap();

                if username_exists {
                    return Err(ApiError::Validation("Username already taken".to_string()));
                }
            }

            // Update the user
            let updated_user =
                diesel::update(users::table.filter(users::address.eq(&req.wallet_address)))
                    .set((
                        users::username.eq(&req.new_username),
                        users::updated.eq(true),
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
