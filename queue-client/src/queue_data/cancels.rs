use serde::{Deserialize, Serialize};
use crate::queue_data::data_core::{QueueData, QueueClass};

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelOrderMessage {
    pub order_id: String,
    pub client_order_id: String
}

impl QueueData for CancelOrderMessage {
    fn class() -> QueueClass {
        QueueClass::Cancel
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelConfirmMessage {
    pub order_id: String,
    pub client_order_id: String
}

impl QueueData for CancelConfirmMessage {
    fn class() -> QueueClass {
        QueueClass::CancelConfirm
    }
}