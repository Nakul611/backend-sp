use crate::{
    auth::{generate_tokens, AuthUser},
    error::AppError,
};

pub fn generate_user_tokens(user: &AuthUser) -> Result<(String, String), AppError> {
    generate_tokens(user)
}
