#![allow(dead_code, unused_imports)]
use axum::{
    http::{Request, Response, Result, StatusCode},
    response::IntoResponse,
    Json,
};
use log::info;
use serde::{Deserialize, Serialize};

pub async fn handler(Json(req): Json<SubReq>) -> impl IntoResponse {
    info!("subscribe handler");
    info!("req: {:?}", req);
    (StatusCode::OK, r#"{"success": true}"#)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubReq {
    pub topic: String,
    pub webhook: String,
}
