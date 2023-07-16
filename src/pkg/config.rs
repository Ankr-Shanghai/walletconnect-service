use log::error;
use std::collections::HashMap;
use std::process::exit;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct AppState {
    pub client: redis::Client,
    pub chan: Arc<Mutex<HashMap<String, Vec<mpsc::Sender<String>>>>>,
    pub sub: HashMap<String, Vec<String>>, // topic -> [addr]
}

pub fn init(addr: String) -> AppState {
    let client = redis::Client::open(addr).unwrap_or_else(|err| {
        error!("redis client init error {}", err);
        exit(-1)
    });
    let chan = Arc::new(Mutex::new(HashMap::new()));
    let sub = HashMap::new();

    AppState { client, chan, sub }
}
impl AppState {
    pub fn save_send_channel(&self, who: String, sender: mpsc::Sender<String>) {
        let mut chan = self.chan.lock().unwrap();
        if let Some(senders) = chan.get_mut(&who) {
            senders.push(sender);
        } else {
            chan.insert(who, vec![sender]);
        }
    }
}
