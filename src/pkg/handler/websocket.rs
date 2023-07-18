#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
use crate::constant;

use super::super::config::AppState;
use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    headers,
    response::IntoResponse,
    TypedHeader,
};
use log4rs::append::Append;
use redis::Commands;
use std::borrow::Cow;
use std::ops::ControlFlow;
use std::sync::Arc;

use log::{error, info};
use std::net::SocketAddr;
//allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown Accessor")
    };
    info!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |socket| handle_socket(socket, addr, app_state))
}

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, who: SocketAddr, app_state: Arc<AppState>) {
    let (tx, mut rx) = mpsc::channel::<String>(8);
    app_state.save_send_channel(who.to_string(), tx).await;

    let (mut sender, mut receiver) = socket.split();
    let mut send_task = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = sender.send(Message::Ping(vec![])).await {
                        error!("from {} send error: {}", who, e);
                        break;
                    }
                }
                Some(msg) = rx.recv() => {
                    if let Err(e) = sender.send(Message::Text(msg)).await {
                        error!("from {} send error: {}", who, e);
                        break;
                    }
                }
            }
        }
    });

    let rx_state = app_state.clone();
    // This second task will receive messages from client and print them on server console
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // print message and break if instructed to do so
            if process_message(msg, who, rx_state.clone()).await.is_break() {
                break;
            }
        }
        cnt
    });

    loop {
        tokio::select! {
            rs = (&mut recv_task) => {
                match rs {
                    Ok(cnt) => info!("from {} received {} messages", who, cnt),
                    Err(e) => error!("from {} receive error: {}", who, e),
                }
                break;
            }

        }
    }

    app_state.remove_send_channel(who.to_string()).await;
    app_state.remove_sub(who.to_string()).await;

    info!("remote connect {} destroyed", who);
}

use super::msg;
/// helper to print contents of messages to stdout. Has special treatment for Close.
async fn process_message(
    msg: Message,
    who: SocketAddr,
    app_state: Arc<AppState>,
) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            info!("from {} message: {}", who, t);
            let msgt: Result<msg::WalletMessage, serde_json::Error> = serde_json::from_str(&t);
            if let Ok(msg) = msgt {
                match msg.kind {
                    msg::MessageKind::Pub => {
                        pub_handler(msg.topic, t, who, app_state.clone()).await
                    }
                    msg::MessageKind::Sub => sub_handler(msg.topic, who, app_state.clone()).await,
                }
            } else {
                error!("from {} sent invalid message: {}", who, t)
            }
        }

        Message::Binary(d) => {
            info!("from {} sent {} bytes", who, d.len());
        }

        Message::Close(c) => {
            if let Some(cf) = c {
                info!(
                    "from {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                info!("from {} somehow sent close message without CloseFrame", who);
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {}

        Message::Ping(v) => {}
    }
    ControlFlow::Continue(())
}

async fn pub_handler(topic: String, msg: String, who: SocketAddr, app_state: Arc<AppState>) {
    if let Some(whos) = app_state.get_sub(topic.clone()).await {
        for who in whos {
            app_state.send_message(who, msg.clone()).await;
        }
    } else {
        let mut conn = app_state.client.get_connection().unwrap();
        if let Ok(()) = conn.lpush(topic.clone(), msg) {
            info!("save topic {} success", topic.clone());
        } else {
            error!("save topic {} failed", topic.clone());
        }
    }
}

async fn sub_handler(topic: String, who: SocketAddr, app_state: Arc<AppState>) {
    app_state.save_sub(topic.clone(), who.to_string()).await;

    let mut conn = app_state.client.get_connection().unwrap();
    let mut cnt = 0;

    loop {
        let msg: Result<Vec<String>, redis::RedisError> = conn.lrange(topic.clone(), 0, -1);
        if let Ok(msgs) = msg {
            if let Ok(()) = conn.del(topic.clone()) {
                info!("del topic {} success", topic.clone())
            }
            for msg in msgs {
                app_state.send_message(who.to_string(), msg).await;
                cnt += 1;
            }
            break;
        } else {
            break;
        }
    }
    info!(
        "from {} sub topic {} success, get {} messages",
        who, topic, cnt
    );
}
