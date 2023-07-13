use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process::exit,
    sync::Arc,
};

use axum::{self, routing::get, routing::post, Router};
use log::{error, info};

#[tokio::main]
async fn main() {
    // init log system
    info!("init log system");
    log4rs::init_file("log_config.yaml", Default::default()).unwrap_or_else(|err| {
        println!("init log error {}", err);
        exit(-1)
    });

    // parse command arguments
    let args = Args::parse();
    info!(
        "start server with {} ...",
        format!("{}:{}", args.host, args.port)
    );

    let state = pkg::config::init(args.redis);
    let state: Arc<pkg::config::AppState> = Arc::new(state);

    // build application with a route
    let app = Router::new()
        // websocket handler
        .route("/", get(pkg::handler::websocket::handler))
        // http handler
        .route("/hello", get(pkg::handler::hello::handler))
        .route("/health", get(pkg::handler::health::handler))
        .route("/info", get(pkg::handler::info::handler))
        .route("/subscribe", post(pkg::handler::subscribe::handler))
        .with_state(state);

    let host = args.host.parse::<IpAddr>().unwrap_or_else(|err| {
        error!("host {} error {} ", args.host, err);
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    });

    let addr = SocketAddr::from((host, args.port));

    let bindr = axum::Server::try_bind(&addr).unwrap_or_else(|err| {
        error!("bind address error {}", err);
        exit(-1)
    });

    bindr
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap_or_else(|err| {
            error!("service boot error {}", err);
            exit(-1)
        });
}

// define command args
use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(short, long, default_value_t = 5200)]
    port: u16,
    #[arg(long, default_value = "redis://127.0.0.1:6379/")]
    redis: String,
}
