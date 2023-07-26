use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    routing::get,
    Router, http::Uri,
};

use std::{net::SocketAddr, str::FromStr};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod channel;
mod client;
use channel::Channel;
use client::Client;

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
        .route("/sub", get(client_handler))
        .route("/pub", get(client_handler))
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

async fn client_handler(ws: WebSocketUpgrade, uri: Uri, State(chan): State<Channel>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, chan, client::ClientRole::from_str(uri.to_string().as_str()).unwrap()))
}

async fn handle_socket(socket: WebSocket, chan: Channel, role: client::ClientRole) {
    let client = Client::new(socket);
    match role {
        client::ClientRole::Publisher => {
            client.publish(&chan).await;
        },
        client::ClientRole::Subscriber => {
            client.subscribe(&chan).await;
        }
    }
}


