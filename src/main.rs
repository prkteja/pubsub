use axum::{
    Json,
    extract::{State, Query, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    routing::{get,post},
    Router, http::StatusCode,
};
use tokio::sync::RwLock;

use std::{net::SocketAddr, str::FromStr, sync::Arc, collections::HashMap};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

mod channel;
mod client;
use channel::Channel;
use client::{Client, ClientRole};


#[derive(Debug, Serialize, Deserialize)]
struct ChannelProps {
    id: Option<Uuid>,
    name: Option<String>,
    size: Option<usize>
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pubsub=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let chan_map: HashMap<Uuid, Channel> = HashMap::new();

    let app = Router::new()
        .route("/createChannel", post(create_channel))
        .route("/getChannels", get(get_channels))
        .route("/attach", get(client_handler))
        .with_state(Arc::new(RwLock::new(chan_map)))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn create_channel(Query(params): Query<ChannelProps>, 
                        State(chan_map): State<Arc<RwLock<HashMap<Uuid, Channel>>>>) -> Result<String, String> {
    if params.name.is_none() {
        return Err("unable to find channel name".to_string());
    }
    let chan = Channel::new(params.name.unwrap(), params.size.unwrap_or(32));
    let chan_id = chan.get_id();
    let mut chan_map = chan_map.write().await;
    chan_map.insert(chan_id, chan);
    Ok(chan_id.to_string())
}

async fn get_channels(State(chan_map): State<Arc<RwLock<HashMap<Uuid, Channel>>>>) -> Json<Vec<ChannelProps>> {
    let mut chan_list = Vec::new();
    for chan in chan_map.read().await.values() {
        chan_list.push(ChannelProps {
            id: Some(chan.get_id()), 
            name: Some(chan.get_name()), 
            size: Some(chan.get_size())
        });
    }
    Json(chan_list)
}

fn parse_uuids(ids: String) -> Option<Vec<Uuid>> {
    let mut uuids: Vec<Uuid> = Vec::new();
    let list = ids.split(",");
    for id in list {
        match Uuid::from_str(id) {
            Ok(id) => {
                uuids.push(id);
            },
            Err(_) => {
                return None;
            }
        }
    }
    Some(uuids)
}

async fn client_handler(ws: WebSocketUpgrade, 
                        Query(params): Query<HashMap<String, String>>,
                        State(chan_map): State<Arc<RwLock<HashMap<Uuid, Channel>>>>) -> impl IntoResponse {
    match params.get("role") {
        None => {
            return (StatusCode::BAD_REQUEST, "unable to find client role").into_response();
        },
        Some(role) => {
            match ClientRole::from_str(role) {
                Ok(client_role) => {
                    if let Some(chan_list) = params.get("channels") {
                        if let Some(id_list) = parse_uuids(chan_list.to_owned()) {
                            return ws.on_upgrade(move |socket| handle_socket(socket, chan_map, id_list, client_role));
                        } else {
                            return (StatusCode::BAD_REQUEST, "unable to parse channel list").into_response();
                        }
                    } else {
                        return (StatusCode::BAD_REQUEST, "unable to find channel list").into_response();
                    }
                },
                Err(error) => {
                    return (StatusCode::BAD_REQUEST, error).into_response();
                }
            }
        }
    }
}

async fn handle_socket(socket: WebSocket, 
                       chan_map: Arc<RwLock<HashMap<Uuid, Channel>>>,
                       id_list: Vec<Uuid>, 
                       role: client::ClientRole) {
    let client = Client::new(socket);
    let mut chan_list: Vec<&Channel> = Vec::new();
    let map = chan_map.read().await;
    for id in id_list.iter() {
        if let Some(chan) = map.get(id) {
            chan_list.push(chan);
        }
    }
    match role {
        client::ClientRole::Publisher => {
            client.bulk_publish(chan_list).await;
        },
        client::ClientRole::Subscriber => {
            client.bulk_subscribe(chan_list).await;
        }
    }
}


