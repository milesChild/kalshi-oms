use core::fmt;
use serde::{Deserialize, de::DeserializeOwned, Serialize};
use anyhow::Result;
use bincode::{serialize, deserialize};

use crate::queue_data::{
    cancels::{CancelConfirmMessage, CancelOrderMessage}, 
    fills::FillMessage, 
    orders::CreateOrderMessage, 
    orders::OrderConfirmMessage
};

/// Types of queue that queue data can be written to
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueueClass {
    Order, 
    OrderConfirm,
    Cancel, 
    CancelConfirm,
    Fill
}

impl fmt::Display for QueueClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueClass::Order => write!(f, "order"),
            QueueClass::Cancel => write!(f, "cancel"),
            QueueClass::OrderConfirm => write!(f, "order_confirm"),
            QueueClass::CancelConfirm => write!(f, "cancel_confirm"),
            QueueClass::Fill => write!(f, "fill")
        }
    }
}

/// A trait all data in Queues must implement
pub trait QueueData: Serialize + DeserializeOwned {
    fn class() -> QueueClass;
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serialize(&self)?)
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self> where Self: Sized {
        Ok(deserialize::<Self>(bytes)?)
    }
}