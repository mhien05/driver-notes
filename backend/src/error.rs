use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub(crate) struct ApiError {
    status: StatusCode,
    message: &'static str,
}

impl ApiError {
    pub(crate) fn bad(message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message,
        }
    }
    pub(crate) fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: "Authentication required",
        }
    }
    pub(crate) fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: "Record not found",
        }
    }
    pub(crate) fn conflict(message: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message,
        }
    }
    pub(crate) fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Internal server error",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}

pub(crate) type ApiResult<T> = Result<T, ApiError>;
