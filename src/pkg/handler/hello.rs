use axum::{http::StatusCode, response::IntoResponse};
pub async fn handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        "Hello World, this is WalletConnect v1.0.0-beta",
    )
}
