use crate::generate_username::generate_random_username;
use crate::model::*;
use crate::schema::users;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
// ...existing code...
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;


#[utoipa::path(
    post,
    path = "/create_user",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created successfully or already exists", body = UserResponse),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "Users"
)]
pub async fn create_user(
    State(pool): State<DbPool>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    let conn = &mut pool.get().expect("Couldn't get db connection from pool");

    // Check if user exists
    let existing_user = match users::table
        .filter(users::address.eq(&req.wallet_address))
        .select(User::as_select())
        .first::<User>(conn)
        .optional()
    {
        Ok(opt) => opt,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Database error: {}", e)})),
            )
                .into_response();
        }
    };

    if let Some(user) = existing_user {
        return (
            StatusCode::OK,
            Json(UserResponse {
                id: user.id,
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
        )
            .into_response();
    }

    if /*req.wallet_address.len() != 65 ||*/ !req.wallet_address.starts_with("0x") {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid Starknet address"})),
        )
            .into_response();
    }

    let new_user = NewUser {
        address: req.wallet_address,
        username: generate_random_username(),
        is_registered: Some(true),
        highest_score: Some(0),
        updated: Some(false),
        registered_at: Some(Utc::now().naive_utc()),
    };

    match diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(conn)
    {
        Ok(user) => (
            StatusCode::OK,
            Json(UserResponse {
                id: user.id,
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
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Error inserting user: {}", e)})),
        )
            .into_response(),
    }
}

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
        (status = 400, description = "Invalid Starknet address", body = String),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "Users"
)]
pub async fn delete_user(
    State(pool): State<DbPool>,
    Json(req): Json<DeleteUserRequest>,
) -> impl IntoResponse {
    let conn = &mut pool.get().expect("Couldn't get db connection from pool");

    // Validate wallet address
    if !req.wallet_address.starts_with("0x") {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid Starknet address"})),
        )
            .into_response();
    }

    // Check if user exists
    let existing_user = match users::table
        .filter(users::address.eq(&req.wallet_address))
        .select(User::as_select())
        .first::<User>(conn)
        .optional()
    {
        Ok(opt) => opt,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Database error: {}", e)})),
            )
                .into_response();
        }
    };

    if let Some(user) = existing_user {
        // Delete the user
        match diesel::delete(users::table.filter(users::address.eq(&req.wallet_address)))
            .execute(conn)
        {
            Ok(_) => (
                StatusCode::OK,
                Json(UserResponse {
                    id: user.id,
                    address: user.address,
                    username: user.username,
                    position: user.position.unwrap_or(0),
                    games_played: user.games_played.unwrap(),
                    is_registered: user.is_registered,
                    highest_score: user.highest_score.unwrap_or(0),
                    updated: user.updated,
                    registered_at: user.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                    status: None,
                }),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Error deleting user: {}", e)})),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        )
            .into_response()
    }
}


#[utoipa::path(
    get,
    path = "/users/{wallet_address}",
    params(
        ("wallet_address" = String, Path, description = "Wallet address of the user", example = "0x1234567890abcdef")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found", body = String)
    ),
    tag = "Users"
)]
pub async fn get_user(
    State(pool): State<DbPool>,
    Path(wallet_address): Path<String>,
) -> impl IntoResponse {
    let conn = &mut pool.get().expect("Couldn't get db connection from pool");

    match users::table
        .filter(users::address.eq(wallet_address))
        .first::<User>(conn)
    {
        Ok(user) => (
            StatusCode::OK,
            Json(UserResponse {
                id: user.id,
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
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "User not found".to_string()).into_response(),
    }
}


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
        (status = 400, description = "Invalid input or username already taken", body = String),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "Users"
)]
pub async fn check_user(
    State(pool): State<DbPool>,
    Json(req): Json<CheckUserRequest>,
) -> impl IntoResponse {
    let conn = &mut pool.get().expect("Couldn't get db connection from pool");

    // Validate wallet address
    if !req.wallet_address.starts_with("0x") {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid Starknet address"})),
        )
            .into_response();
    }

    // Validate username
    if req.username.is_empty() || req.username.len() > 50 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Username must be between 1 and 50 characters"})),
        )
            .into_response();
    }

    if !req.username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Username must contain only alphanumeric characters or underscores"})),
        )
            .into_response();
    }

    // Check if username is already taken
    let username_exists = match users::table
        .filter(users::username.eq(&req.username))
        .select(users::id)
        .first::<i32>(conn)
        .optional()
    {
        Ok(opt) => opt,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Database error: {}", e)})),
            )
                .into_response();
        }
    };

    if username_exists.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Username already taken"})),
        )
            .into_response();
    }

    // Check if user exists by wallet address
    let user = match users::table
        .filter(users::address.eq(&req.wallet_address))
        .select(User::as_select())
        .first::<User>(conn)
        .optional()
    {
        Ok(opt) => opt,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Database error: {}", e)})),
            )
                .into_response();
        }
    };

    match user {
        Some(user) => (
            StatusCode::OK,
            Json(UserResponse {
                id: user.id,
                address: user.address,
                username: user.username,
                position: user.position.unwrap_or(0),
                games_played: user.games_played.unwrap(),
                is_registered: user.is_registered,
                highest_score: user.highest_score.unwrap_or(0),
                updated: user.updated,
                registered_at: user.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                status: Some(UserStatus::Existing),
            }),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        )
            .into_response(),
    }
}


#[utoipa::path(
    put,
    path = "/update_user",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Username updated successfully", body = UserResponse),
        (status = 400, description = "Invalid input, update limit reached, or username already taken", body = String),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "Users"
)]
pub async fn update_user(
    State(pool): State<DbPool>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    let conn = &mut pool.get().expect("Couldn't get db connection from pool");

    // Validate wallet address
    if !req.wallet_address.starts_with("0x") {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid Starknet address"})),
        )
            .into_response();
    }

    // Validate username
    if req.new_username.is_empty() || req.new_username.len() > 50 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Username must be between 1 and 50 characters"})),
        )
            .into_response();
    }

    if !req.new_username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Username must contain only alphanumeric characters or underscores"})),
        )
            .into_response();
    }

    // Check if user exists and get update status
    let user = users::table
        .filter(users::address.eq(&req.wallet_address))
        .select((users::updated, users::id, users::username))
        .first::<(bool, i32, String)>(conn)
        .optional()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Database error: {}", e)})),
            )
                .into_response()
        })
        .unwrap();

    match user {
        Some((true, _, _)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Username update limit reached: only one update allowed"})),
            )
                .into_response();
        }
    Some((_, _id, current_username)) => {
            // Check if new_username is already taken by another user
            if current_username != req.new_username {
                let username_exists = match users::table
                    .filter(users::username.eq(&req.new_username))
                    // .filter(users::address.ne(&req.wallet_address))
                    .select(users::id)
                    .first::<i32>(conn)
                    .optional()
                {
                    Ok(opt) => opt,
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": format!("Database error: {}", e)})),
                        )
                            .into_response();
                    }
                };

                if username_exists.is_some() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Username already taken"})),
                    )
                        .into_response();
                }
            }
        }
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "User not found"})),
            )
                .into_response();
        }
    }

    // Update the user
    match diesel::update(users::table.filter(users::address.eq(&req.wallet_address)))
        .set((
            users::username.eq(&req.new_username),
            users::updated.eq(true),
        ))
        .get_result::<User>(conn)
    {
        Ok(user) => (
            StatusCode::OK,
            Json(UserResponse {
                id: user.id,
                address: user.address,
                username: user.username,
                position: user.position.unwrap_or(0),
                games_played: user.games_played.unwrap(),
                is_registered: user.is_registered,
                highest_score: user.highest_score.unwrap_or(0),
                updated: user.updated,
                registered_at: user.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                status: None,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Error updating user: {}", e)})),
        )
            .into_response(),
    }
}


//==========
//
//
// #[derive(Serialize, Deserialize, ToSchema)]
// pub struct SubmitScoreRequest {
//     pub wallet_address: String,
//     pub score: i32,
//     pub game_duration: i32,
// }
//
// #[derive(Serialize, ToSchema)]
// pub struct SubmitScoreResponse {
//     pub total_score: i32,
//     pub games_played: i32,
//     pub highest_score: i32,
// }
//
// #[derive(Serialize, ToSchema)]
// pub struct GameScoreResponse {
//     pub id: i32,
//     pub user_id: i32,
//     pub wallet_address: String,
//     pub score: i32,
//     pub game_duration: i32,
//     pub played_at: String,
// }
//
// #[derive(Deserialize, ToSchema)]
// pub struct RecentGamesQuery {
//     pub limit: Option<i64>,
// }
//
// #[derive(Insertable, Queryable, Selectable)]
// #[diesel(table_name = game_scores)]
// pub struct GameScore {
//     pub id: i32,
//     pub user_id: i32,
//     pub wallet_address: String,
//     pub score: i32,
//     pub game_duration: i32,
//     pub played_at: NaiveDateTime,
// }
//
// #[utoipa::path(
//     post,
//     path = "/submit_score",
//     request_body = SubmitScoreRequest,
//     responses(
//         (status = 200, description = "Score submitted successfully", body = SubmitScoreResponse),
//         (status = 400, description = "Invalid input", body = String),
//         (status = 404, description = "User not found", body = String),
//         (status = 500, description = "Internal server error", body = String)
//     ),
//     tag = "Game"
// )]
// pub async fn submit_score(
//     State(pool): State<DbPool>,
//     Json(req): Json<SubmitScoreRequest>,
// ) -> impl IntoResponse {
//     let conn = &mut pool.get().expect("Couldn't get db connection from pool");
//
//     // Validate wallet address
//     if req.wallet_address.len() != 66 || !req.wallet_address.starts_with("0x") {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Invalid Starknet address"})),
//         )
//             .into_response();
//     }
//
//     // Validate score
//     if req.score < 0 {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Score must be non-negative"})),
//         )
//             .into_response();
//     }
//
//     // Validate game duration
//     if req.game_duration < 0 {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Game duration must be non-negative"})),
//         )
//             .into_response();
//     }
//
//     // Check if user exists
//     let user = users::table
//         .filter(users::address.eq(&req.wallet_address))
//         .select(User::as_select())
//         .first(conn)
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
//     let user = match user {
//         Some(user) => user,
//         None => {
//             return (
//                 StatusCode::NOT_FOUND,
//                 Json(serde_json::json!({"error": "User not found"})),
//             )
//                 .into_response();
//         }
//     };
//
//     // Insert game score
//     let new_score = GameScore {
//         id: 0, // Will be set by SERIAL
//         user_id: user.id,
//         wallet_address: req.wallet_address.clone(),
//         score: req.score,
//         game_duration: req.game_duration,
//         played_at: Utc::now().naive_utc(),
//     };
//
//     let score_result = diesel::insert_into(game_scores::table)
//         .values(&new_score)
//         .execute(conn)
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(serde_json::json!({"error": format!("Error inserting score: {}", e)})),
//             )
//                 .into_response()
//         });
//
//     if score_result.is_err() {
//         return score_result.unwrap_err();
//     }
//
//     // Update user stats
//     let new_total_score = user.highest_score + req.score;
//     let new_games_played = user.games_played + 1;
//     let new_highest_score = user.highest_score.unwrap_or(0).max(req.score);
//
//     let update_result = diesel::update(users::table.filter(users::address.eq(&req.wallet_address)))
//         .set((
//             users::highest_score.eq(new_total_score),
//             users::games_played.eq(new_games_played),
//             users::highest_score.eq(Some(new_highest_score)),
//             users::updated_at.eq(Some(Utc::now().naive_utc())),
//         ))
//         .execute(conn)
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(serde_json::json!({"error": format!("Error updating user: {}", e)})),
//             )
//                 .into_response()
//         });
//
//     if update_result.is_err() {
//         return update_result.unwrap_err();
//     }
//
//     (
//         StatusCode::OK,
//         Json(SubmitScoreResponse {
//             total_score: new_total_score,
//             games_played: new_games_played,
//             highest_score: new_highest_score,
//         }),
//     )
//         .into_response()
// }
//
// #[utoipa::path(
//     get,
//     path = "/player_stats/{wallet_address}",
//     params(
//         ("wallet_address" = String, Path, description = "Wallet address of the user", example = "0x1234567890abcdef")
//     ),
//     responses(
//         (status = 200, description = "Player stats found", body = UserResponse),
//         (status = 400, description = "Invalid Starknet address", body = String),
//         (status = 404, description = "User not found", body = String),
//         (status = 500, description = "Internal server error", body = String)
//     ),
//     tag = "Game"
// )]
// pub async fn get_player_stats(
//     State(pool): State<DbPool>,
//     Path(wallet_address): Path<String>,
// ) -> impl IntoResponse {
//     let conn = &mut pool.get().expect("Couldn't get db connection from pool");
//
//     // Validate wallet address
//     if wallet_address.len() != 66 || !wallet_address.starts_with("0x") {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Invalid Starknet address"})),
//         )
//             .into_response();
//     }
//
//     match users::table
//         .filter(users::address.eq(&wallet_address))
//         .select(User::as_select())
//         .first(conn)
//         .optional()
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(serde_json::json!({"error": format!("Database error: {}", e)})),
//             )
//                 .into_response()
//         })
//         .unwrap()
//     {
//         Some(user) => (
//             StatusCode::OK,
//             Json(UserResponse {
//                 id: user.id,
//                 address: user.address,
//                 username: user.username,
//                 position: user.position.unwrap_or(0),
//                 is_registered: user.is_registered,
//                 highest_score: user.highest_score.unwrap_or(0),
//                 games_played: user.games_played.unwrap(),
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

// #[utoipa::path(
//     get,
//     path = "/recent_games/{wallet_address}",
//     params(
//         ("wallet_address" = String, Path, description = "Wallet address of the user", example = "0x1234567890abcdef"),
//         ("limit" = i64, Query, description = "Number of recent games to fetch", example = "10")
//     ),
//     responses(
//         (status = 200, description = "Recent games found", body = Vec<GameScoreResponse>),
//         (status = 400, description = "Invalid Starknet address or limit", body = String),
//         (status = 404, description = "No games found", body = String),
//         (status = 500, description = "Internal server error", body = String)
//     ),
//     tag = "Game"
// )]
// pub async fn get_recent_games(
//     State(pool): State<DbPool>,
//     Path(wallet_address): Path<String>,
//     Query(params): Query<RecentGamesQuery>,
// ) -> impl IntoResponse {
//     let conn = &mut pool.get().expect("Couldn't get db connection from pool");
//
//     // Validate wallet address
//     if wallet_address.len() != 66 || !wallet_address.starts_with("0x") {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Invalid Starknet address"})),
//         )
//             .into_response();
//     }
//
//     // Validate limit
//     let limit = params.limit.unwrap_or(10);
//     if limit < 1 || limit > 100 {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "Limit must be between 1 and 100"})),
//         )
//             .into_response();
//     }
//
//     let games = game_scores::table
//         .filter(wallet_address.eq(&wallet_address))
//         .order(game_scores::played_at.desc())
//         .limit(limit)
//         .select(GameScore::as_select())
//         .load::<GameScore>(conn)
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(serde_json::json!({"error": format!("Database error: {}", e)})),
//             )
//                 .into_response()
//         })
//         .unwrap();
//
//     if games.is_empty() {
//         return (
//             StatusCode::NOT_FOUND,
//             Json(serde_json::json!({"error": "No games found"})),
//         )
//             .into_response();
//     }
//
//     (
//         StatusCode::OK,
//         Json(games.into_iter().map(|game| GameScoreResponse {
//             id: game.id,
//             user_id: game.user_id,
//             wallet_address: game.wallet_address,
//             score: game.score,
//             game_duration: game.game_duration,
//             played_at: game.played_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
//         }).collect::<Vec<GameScoreResponse>>()),
//     )
//         .into_response()
// }