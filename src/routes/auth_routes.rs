use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    controllers::auth_controller::{get_current_user, login, logout, refresh_token_handler},
    state::AppState,
};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh_token_handler))
        .route("/me", get(get_current_user))
        .route("/logout", post(logout))
}
