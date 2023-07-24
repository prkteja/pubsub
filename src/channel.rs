use uuid::Uuid;
use tokio::sync::broadcast;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Channel {
    id: Uuid,
    size: usize,
    tx: broadcast::Sender<String>,
}

#[allow(dead_code)]
impl Channel {
    pub fn new(size: usize) -> Self {
        let (tx, _) = broadcast::channel(size);
        Channel {
            id: Uuid::new_v4(),
            size,
            tx
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_tx(&self) -> broadcast::Sender<String> {
        self.tx.to_owned()
    }

    pub fn get_rx(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

