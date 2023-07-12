use log::error;
use std::process::exit;

pub struct AppState {
    pub client: redis::Client,
}

pub fn init(addr: String) -> AppState {
    let client = redis::Client::open(addr).unwrap_or_else(|err| {
        error!("redis client init error {}", err);
        exit(-1)
    });
    AppState { client }
}
