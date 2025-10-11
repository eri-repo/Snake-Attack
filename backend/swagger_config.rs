use crate::functions::{__path_check_user, __path_create_user, __path_delete_user, __path_get_user,
                       __path_update_user};
use crate::model::{CreateUserRequest, UpdateUserRequest, UserResponse};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(create_user, get_user, update_user, delete_user, check_user),
    components(schemas(CreateUserRequest, UserResponse, UpdateUserRequest)),
    tags((name = "Starknake", description = "Starknake endpoints")),
    info(
        title = "Starknake",
        description = "Snake game on Starknet",
    ),
)]
pub struct ApiDoc;