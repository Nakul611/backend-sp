use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data,
            message: None,
        })
    }

    pub fn with_message(data: T, message: impl Into<String>) -> Json<Self> {
        Json(Self {
            success: true,
            data,
            message: Some(message.into()),
        })
    }
}
