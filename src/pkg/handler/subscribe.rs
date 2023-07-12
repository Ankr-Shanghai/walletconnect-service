#![allow(dead_code, unused_imports, unused_variables)]
use super::super::config::AppState;
use axum::{
    extract::State,
    http::{Request, Response, Result, StatusCode},
    response::IntoResponse,
    Json,
};
use log::info;
use redis::{Client, Commands};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn handler(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<SubReq>,
) -> impl IntoResponse {
    info!("subscribe handler");
    info!("req: {:?}", req);
    let mut client = app_state.client.get_connection().unwrap();
    let val: String = client.get("hello").unwrap();
    info!("val: {:?}", val);

    (StatusCode::OK, r#"{"success": true}"#)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubReq {
    pub topic: String,
    pub webhook: String,
}
