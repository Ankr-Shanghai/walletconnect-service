#![allow(dead_code, unused_imports, unused_variables)]
use super::super::config::AppState;
use super::super::constant;
use axum::{
    extract::State,
    http::{Request, Response, Result, StatusCode},
    response::IntoResponse,
    Json,
};
use clap::error;
use log::{error, info};
use redis::{Client, Commands};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn handler(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<SubReq>,
) -> impl IntoResponse {
    info!("subscribe handler");
    info!("req: {:?}", req);

    let reqs = serde_json::to_string(&req).unwrap();
    if let Ok(mut client) = app_state.client.get_connection() {
        if let Ok(_) = client.set_ex::<String, String, usize>(req.topic, reqs, constant::TIMEOUT) {
            info!("save webhook into cache success");
        } else {
            error!("save webhook into cahce failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, r#"{"success": false}"#);
        }
    } else {
        error!("get redis connection error");
        return (StatusCode::INTERNAL_SERVER_ERROR, r#"{"success": false}"#);
    }

    (StatusCode::OK, r#"{"success": true}"#)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubReq {
    pub topic: String,
    pub webhook: String,
}
