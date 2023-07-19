use uuid::Uuid;
use tokio::sync::broadcast;

pub struct Channel {
    pub id: Uuid,
    pub size: usize,
    pub tx: broadcast::Sender<String>,
    pub rx: broadcast::Receiver<String>
}

impl Channel {
    pub fn new(size: usize) -> Self {
        let (tx, rx) = broadcast::channel(size);
        Channel {
            id: Uuid::new_v4(),
            size: 32,
            tx,
            rx
        }
    }
}

impl Clone for Channel {
    fn clone(&self) -> Self {
        Channel {
            id: self.id.to_owned(),
            size: self.size.to_owned(), 
            tx: self.tx.to_owned(), 
            rx: self.tx.subscribe() 
        }
    }
}
