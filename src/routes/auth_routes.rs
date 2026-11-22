use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    controllers::auth_controller::{get_current_user, login, logout, refresh_token_handler},
    state::AppState,
};

pub fn public_auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh_token_handler))
}

pub fn protected_auth_routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_current_user))
        .route("/logout", post(logout))
}

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .merge(public_auth_routes())
        .merge(protected_auth_routes())
}
