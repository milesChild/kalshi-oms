use serde::{Serialize, Deserialize};

use crate::queue_data::data_core::{QueueData, QueueClass};

use kalshi::Action;
use kalshi::Side;
use kalshi::OrderType;

extern crate kalshi;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateOrderMessage {
    pub action: Action,
    pub client_order_id: String,
    pub count: i32,
    pub side: Side,
    pub ticker: String,
    pub input_type: OrderType,
    pub buy_max_cost: Option<i64>,
    pub expiration_ts: Option<i64>,
    pub no_price: Option<i64>,
    pub sell_position_floor: Option<i32>,
    pub yes_price: Option<i64>
}

impl QueueData for CreateOrderMessage {
    fn class() -> QueueClass {
        QueueClass::Order
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderConfirmMessage {
    order_id: String,
    client_order_id: Option<String>
}

impl QueueData for OrderConfirmMessage {
    fn class() -> QueueClass {
        QueueClass::OrderConfirm
    }
}

impl OrderConfirmMessage {
    pub fn new(order_id: String, client_order_id: Option<String>) -> Self {
        OrderConfirmMessage {
            order_id,
            client_order_id,
        }
    }
}