use tokio::sync::broadcast;
use uuid::Uuid;
use axum::extract::ws::{WebSocket, Message};
use tracing::{self, debug, info, warn, error};

pub enum ClientRole {
    Publisher,
    Subscriber
}

pub struct Subscriber {
    id: Uuid,
    ws: WebSocket,
    chan_id: Uuid,
    rx: broadcast::Receiver<String>
}

impl Subscriber {
    pub fn new(ws: WebSocket, chan_id: Uuid, rx: broadcast::Receiver<String>) -> Self {
        Subscriber { 
            id: Uuid::new_v4(), 
            ws, 
            chan_id,
            rx 
        }
    }

    // listen for messages on channel and transmit them over the WebSocket
    pub async fn attach(&mut self) {
        while let Ok(msg) = self.rx.recv().await {
            debug!("Got message {} from channel {}", msg, self.chan_id);
            if self.ws.send(Message::Text(msg)).await.is_err() {
                warn!("Client {} abruptly disconnected", self.id);
                return;
            }
        }
    }
}

pub struct Publisher {
    id: Uuid,
    ws: WebSocket,
    chan_id: Uuid,
    tx: broadcast::Sender<String>
}

impl Publisher {
    pub fn new(ws: WebSocket, chan_id: Uuid, tx: broadcast::Sender<String>) -> Self {
        Publisher { 
            id: Uuid::new_v4(), 
            ws, 
            chan_id, 
            tx 
        }
    }

    // listen for messages on WebSocket and enque them on the channel
    pub async fn attach(&mut self) {
        loop {
            if let Some(msg) = self.ws.recv().await {
                if let Ok(msg) = msg {
                    match msg {
                        Message::Text(t) => {
                            debug!("Received {} from client {}", t, self.id);
                            match self.tx.send(t.clone()) {
                                Ok(_) => {
                                    debug!("Pushed {} to channel {}", t, self.chan_id);
                                },
                                Err(e) => {
                                    error!("Message {} failed to enque on channel {}, Error: {}", t, self.chan_id, e);      
                                }
                            };
                        },
                        Message::Close(_) => {
                            info!("Client {} disconnected", self.id);
                            return;
                        },
                        _ => {
                            warn!("Invalid message received from client {}", self.id);
                        }
                    }
                } else {
                    warn!("Client {} abruptly disconnected", self.id);
                    return;
                }
            }
        }
    }
}
