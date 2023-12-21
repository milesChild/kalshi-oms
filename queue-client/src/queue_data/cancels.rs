use serde::{Deserialize, Serialize};
use crate::queue_data::data_core::{QueueData, QueueClass};

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelOrderMessage {
    order_id: String,
    client_order_id: String
}

impl QueueData for CancelOrderMessage {
    fn class() -> QueueClass {
        QueueClass::Cancel
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelConfirmMessage {
    order_id: String,
    client_order_id: String
}

impl QueueData for CancelConfirmMessage {
    fn class() -> QueueClass {
        QueueClass::CancelConfirm
    }
}