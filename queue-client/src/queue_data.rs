use serde::{Deserialize, Serialize};
use anyhow::Result;
use kalshi::Order;
use bincode::{serialize, deserialize};

/// Types of queue that queue data can be written to
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueueClass {
    ORDER, 
    ORDERCONFIRM,
    CANCEL, 
    CANCELCONFIRM,
    FILL
}

/// A trait all data in Queues must implement
pub trait QueueData {
    fn class() -> QueueClass;
    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(bytes: &[u8]) -> Result<Self> where Self: Sized;
}

impl QueueData for Order {

    fn class() -> QueueClass {
        QueueClass::ORDER
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(serialize(&self)?)
    }

    fn deserialize(bytes: &[u8]) -> Result<Self> {
        Ok(deserialize::<Order>(bytes)?)
    }
}