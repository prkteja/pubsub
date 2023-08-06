use uuid::Uuid;
use tokio::sync::broadcast;

#[allow(dead_code)]
pub struct Channel {
    id: Uuid,
    name: String,
    size: usize,
    tx: broadcast::Sender<String>,
    rx: broadcast::Receiver<String>
}

#[allow(dead_code)]
impl Channel {
    pub fn new(name: String, size: usize) -> Self {
        let (tx, rx) = broadcast::channel(size);
        Channel {
            id: Uuid::new_v4(),
            name,
            size,
            tx,
            rx
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
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

