use crate::handlers::{
    check_user::__path_check_user,
    create_user::__path_create_user, delete_user::__path_delete_user, get_user::__path_get_user,
    leaderboard::__path_leaderboard, update_score::__path_update_score, update_user::__path_update_user,
};
use crate::model::{CreateUserRequest, UpdateUserRequest, UserResponse};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_user, delete_user, check_user, update_user, create_user, update_score, leaderboard
    ),
    components(schemas(CreateUserRequest, UserResponse, UpdateUserRequest)),
    tags((name = "Snake Attack", description = "Snake Attack endpoints")),
    info(
        title = "Snake Attack",
        description = "Snake game on Starknet",
    ),
)]
pub struct ApiDoc;
