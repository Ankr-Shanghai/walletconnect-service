use log::error;
use std::collections::HashMap;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct AppState {
    pub client: redis::Client,
    pub chan: Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>,
    pub sub: Arc<Mutex<HashMap<String, Vec<String>>>>, // topic -> [addr]
    pub topic: Arc<Mutex<HashMap<String, Vec<String>>>>, // addr -> [topic]
}

pub fn init(addr: String) -> AppState {
    let client = redis::Client::open(addr).unwrap_or_else(|err| {
        error!("redis client init error {}", err);
        exit(-1)
    });
    let chan: Arc<Mutex<HashMap<String, mpsc::Sender<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let sub: Arc<Mutex<HashMap<String, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));
    let topic: Arc<Mutex<HashMap<String, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));

    AppState {
        client,
        chan,
        sub,
        topic,
    }
}
impl AppState {
    pub async fn save_send_channel(&self, who: String, sender: mpsc::Sender<String>) {
        let mut chan = self.chan.lock().await;
        chan.insert(who, sender);
    }

    pub async fn remove_send_channel(&self, who: String) {
        let mut chan = self.chan.lock().await;
        chan.remove(&who);
    }

    pub async fn send_message(&self, who: String, msg: String) {
        let mut chan = self.chan.lock().await;
        if let Some(senders) = chan.get_mut(&who) {
            if let Err(e) = senders.send(msg.clone()).await {
                error!("send message error: {}", e);
            }
        }
    }

    pub async fn save_sub(&self, topic: &str, who: &str) {
        let mut sub = self.sub.lock().await;
        if let Some(whos) = sub.get_mut(topic) {
            whos.push(who.to_string());
        } else {
            sub.insert(topic.to_string(), vec![who.to_string()]);
        }

        let mut topic_who = self.topic.lock().await;
        if let Some(topics) = topic_who.get_mut(who) {
            topics.push(topic.to_string());
        } else {
            topic_who.insert(who.to_string(), vec![topic.to_string()]);
        }
    }

    pub async fn get_sub(&self, topic: &str) -> Option<Vec<String>> {
        let sub = self.sub.lock().await;
        if let Some(whos) = sub.get(topic) {
            Some(whos.clone())
        } else {
            None
        }
    }

    pub async fn remove_sub(&self, who: &str) {
        let mut topic_who = self.topic.lock().await;
        if let Some(topics) = topic_who.get_mut(who) {
            for topic in topics.iter() {
                let mut sub = self.sub.lock().await;
                if let Some(whos) = sub.get_mut(topic) {
                    whos.retain(|x| x != &who);
                }
            }
        }
    }
}
