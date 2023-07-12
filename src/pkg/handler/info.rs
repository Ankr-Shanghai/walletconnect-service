use log::error;
use serde::{Deserialize, Serialize};
use std::result::Result;
#[derive(Serialize, Deserialize)]
pub struct InfoRsp {
    pub name: String,
    pub version: String,
    pub description: String,
}

pub async fn handler() -> Result<Vec<u8>, ()> {
    let rs = InfoRsp {
        name: "Rust WalletConnect Service".to_string(),
        version: "1.0.0-beta".to_string(),
        description: "WalletConnect Bridge Server".to_string(),
    };
    let rsp = serde_json::to_vec(&rs).unwrap_or_else(|err| {
        error!("serialize error {}", err);
        vec![]
    });
    Ok(rsp)
}
