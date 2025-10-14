use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}

// users (id) {
// id -> Int4,
// address -> Varchar,
// username -> Varchar,
// position -> Nullable<Int8>,
// is_registered -> Bool,
// highest_score -> Nullable<Int8>,
// games_played -> Nullable<Int8>,
// updated -> Bool,
// registered_at -> Timestamp,
// }
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub address: String,
    pub username: String,
    pub position: Option<i64>,
    pub is_registered: bool,
    pub highest_score: Option<i64>,
    pub games_played: Option<i64>,
    pub updated: bool,
    pub registered_at: NaiveDateTime,
}


#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub address: String,
    pub username: String,
    pub is_registered: bool,
    pub highest_score: Option<i64>,
    pub updated: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    #[schema(example = "0x1234567890abcdef")]
    pub(crate) wallet_address: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum UserStatus {
    New,
    Existing,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    #[schema(example = "0x1234567890123456789012345678901234567890123456789012345678901234")]
    #[serde(rename = "walletAddress")]
    pub address: String,
    #[schema(example = "SwiftMage4721")]
    pub username: String,
    #[schema(example = "1")]
    pub position: i64,
    pub games_played: i64,
    #[schema(example = true)]
    pub is_registered: bool,
    #[schema(example = "0")]
    pub highest_score: i64,
    #[schema(example = false)]
    pub updated: bool,
    #[schema(example = "2025-08-14T12:41:00")]
    pub registered_at: String,
    pub status: Option<UserStatus>,

}

#[derive(serde::Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    #[schema(example = "0x1234567890123456789012345678901234567890123456789012345678901234")]
    pub wallet_address: String,
    #[schema(example = "CoolDragon123")]
    pub new_username: String,
}