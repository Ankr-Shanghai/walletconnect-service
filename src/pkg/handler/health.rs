use axum::{http::StatusCode, response::IntoResponse};
pub async fn handler() -> impl IntoResponse {
    (StatusCode::NO_CONTENT, "")
}
