use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    routing::get,
    Router,
};

use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod channel;
mod client;
use channel::Channel;
use client::Subscriber;
use client::Publisher;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pubsub=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let chan = Channel::new(32);
    let app = Router::new()
        .route("/sub", get(sub_handler))
        .route("/pub", get(pub_handler))
        .with_state(chan)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn sub_handler(ws: WebSocketUpgrade, State(chan): State<Channel>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, chan, client::ClientRole::Subscriber))
}
async fn pub_handler(ws: WebSocketUpgrade, State(chan): State<Channel>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, chan, client::ClientRole::Publisher))
}

async fn handle_socket(socket: WebSocket, chan: Channel, role: client::ClientRole) {
    match role {
        client::ClientRole::Publisher => {
            // publish here
            let mut publ: Publisher = Publisher::new(socket, chan.id, chan.tx);
            publ.attach().await;
        },
        client::ClientRole::Subscriber => {
            let mut sub: Subscriber = Subscriber::new(socket, chan.id, chan.rx);
            sub.attach().await;
        }
    }
}


