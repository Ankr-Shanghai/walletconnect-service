#![allow(unused_imports, unused_variables, unused_mut)]
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
    info!("websocket handler");

    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown Accessor")
    };
    info!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, app_state))
}

// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, who: SocketAddr, app_state: Arc<AppState>) {
    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).

    let (mut sender, mut receiver) = socket.split();

    // Spawn a task that will push several messages to the client (does not matter what client does)
    let mut send_task = tokio::spawn(async move {
        // In case of any websocket error, we exit.
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            // ping remote client
            let rs = sender.send(Message::Ping(vec![])).await;
            match rs {
                Ok(_) => {
                    info!("send task to {} ping", who);
                }
                Err(e) => {
                    error!("Websocket context {} send error: {}", who, e);
                    break;
                }
            }
        }
    });

    // This second task will receive messages from client and print them on server console
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // print message and break if instructed to do so
            if process_message(msg, who).is_break() {
                break;
            }
        }
        cnt
    });

    loop {
        // If any one of the tasks exit, abort the other.
        tokio::select! {
            rs = (&mut recv_task) => {
                match rs {
                    Ok(cnt) => info!("Websocket context {} received {} messages", who, cnt),
                    Err(e) => error!("Websocket context {} receive error: {}", who, e),
                }
                break;
            }
        }
    }

    // returning from the handler closes the websocket connection
    // info!("Websocket context {} destroyed", who);
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            info!(">>> {} sent str: {:?}", who, t);
        }
        Message::Binary(d) => {
            info!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                info!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                info!(">>> {} somehow sent close message without CloseFrame", who);
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            info!(">>> {} sent pong with {:?}", who, v);
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            info!(">>> {} sent ping with {:?}", who, v);
        }
    }
    ControlFlow::Continue(())
}
