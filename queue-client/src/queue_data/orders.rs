use serde::{Serialize, Deserialize};

use crate::queue_data::data_core::{QueueData, QueueClass};
use crate::queue_data::data_core::{Action, Side, OrderType};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateOrderMessage {
    action: Action,
    client_order_id: String,
    count: i32,
    side: Side,
    ticker: String,
    input_type: OrderType,
    buy_max_cost: Option<i64>,
    expiration_ts: Option<i64>,
    no_price: Option<i64>,
    sell_position_floor: Option<i32>,
    yes_price: Option<i64>
}

impl QueueData for CreateOrderMessage {
    fn class() -> QueueClass {
        QueueClass::Order
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderConfirmMessage {
    pub order_id: String,
    pub client_order_id: Option<String>
}

impl QueueData for OrderConfirmMessage {
    fn class() -> QueueClass {
        QueueClass::OrderConfirm
    }
}