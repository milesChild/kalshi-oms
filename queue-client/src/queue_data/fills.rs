use serde::{Deserialize, Serialize};

use crate::queue_data::data_core::{QueueClass, QueueData};

use kalshi::{Action, Side};


/// A kalshi market data message
#[derive(Serialize, Deserialize)]
pub struct FillMessage {
    #[serde(rename="type")]
    msg_type: String,
    sid: u32,
    seq: u32,
    pub msg: Fill
}

/// A fill message, i.e. a message containing a fill that has
/// occurred on a ticker
#[derive(Serialize, Deserialize, Debug)]
pub struct Fill {
    pub trade_id: String,
    pub order_id: String,
    pub market_ticker: String,
    pub is_taker: bool,
    pub side: Side,
    pub yes_price: i32,
    pub no_price: i32,
    pub count: i32,
    pub action: Action,
    pub ts: i64
}

impl QueueData for FillMessage {
    fn class() -> QueueClass {
        QueueClass::Fill
    }
}