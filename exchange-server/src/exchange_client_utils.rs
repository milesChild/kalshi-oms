use serde::{Deserialize, Serialize};
use crate::kalshi_wss::Action;

use crate::kalshi_wss::Side;
use kalshi::OrderType;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateOrderMessage {
    action: Action,
    client_order_id: Option<String>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelOrderMessage {
    order_id: String
}