use std::sync::Arc;

use futures::future::join_all;
use uuid::Uuid;
use axum::extract::ws::{WebSocket, Message};
use tracing::{self, debug, info, warn, error};
use tokio::sync::Mutex;
use super::Channel;

pub enum ClientRole {
    Publisher,
    Subscriber
}

pub struct Client {
    id: Uuid,
    ws: Arc<Mutex<WebSocket>>
}

impl Client {
    pub fn new(ws: WebSocket) -> Self {
        Client { 
            id: Uuid::new_v4(), 
            ws: Arc::new(Mutex::new(ws)) 
        }
    }

    pub async fn subscribe(&self, chan: &Channel) {
        while let Ok(msg) = chan.get_rx().recv().await {
            debug!("Got message {} from channel {}", msg, chan.get_id());
            let mut soc = self.ws.lock().await;
            if  soc.send(Message::Text(msg)).await.is_err() {
                warn!("Client {} abruptly disconnected", self.id);
                return;
            }
        }
    }

    pub async fn bulk_subscribe(&self, chan_list: Vec<&Channel>) {
        let fut_list = chan_list.into_iter().map(|chan| self.subscribe(chan));
        join_all(fut_list).await;
    }

    async fn enque_msg(&self, msg: Message, chan: &Channel) {
        match msg {
            Message::Text(t) => {
                debug!("Received {} from client {}", t, self.id);
                match chan.get_tx().send(t.clone()) {
                    Ok(_) => {
                        debug!("Pushed {} to channel {}", t, chan.get_id());
                    },
                    Err(e) => {
                        error!("Message {} failed to enque on channel {}, Error: {}", t, chan.get_id(), e);      
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
    }

    pub async fn publish(&self, chan: &Channel) {
        let mut soc = self.ws.lock().await;
        loop {
            if let Some(msg) = soc.recv().await {
                if let Ok(msg) = msg {
                    self.enque_msg(msg, chan).await;
                } else {
                    warn!("Client {} abruptly disconnected", self.id);
                    return;
                }
            }
        }
    }
    
    pub async fn bulk_publish(&self, chan_list: Vec<&Channel>) {
        let mut soc = self.ws.lock().await;
        loop {
            if let Some(msg) = soc.recv().await {
                if let Ok(msg) = msg {
                    for chan in chan_list.iter() {
                        self.enque_msg(msg.to_owned(), *chan).await;
                    }
                } else {
                    warn!("Client {} abruptly disconnected", self.id);
                    return;
                }
            }
        }
    }
}
