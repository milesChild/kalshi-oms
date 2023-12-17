use log::debug;
use serde_json;
use serde::{Deserialize, Serialize};
use websocket::Message;
use anyhow::anyhow;

use std::ops;

/// Represents a message to send to the Kalshi websocket server.
#[derive(Serialize, Deserialize)]
pub struct KalshiClientMessage {
    id: u32,
    cmd: String,
    params: KalshiClientSubMessage
}

impl KalshiClientMessage {

    /// Convert this client message to a websocket Message
    pub fn to_websocket_message(&self) -> Message {
        Message::text(serde_json::to_string(&self).unwrap())
    }
}

/// A builder for KalshiClientMessages that keeps track of the 
/// id to use for each subsequent message.
pub struct KalshiClientMessageBuilder {
    next_id: u32,
    cmd: Option<String>,
    params: Option<KalshiClientSubMessage>
}

impl KalshiClientMessageBuilder {

    /// Construct a new builder
    pub fn new() -> KalshiClientMessageBuilder {
        KalshiClientMessageBuilder {
            next_id: 1, 
            cmd: None,
            params: None
        }
    }

    /// Set the SubMessage for the next message to build.
    pub fn content(&mut self, submsg: KalshiClientSubMessage) -> &mut Self {
        match submsg {
            KalshiClientSubMessage::SubscribeSubMessage(ref _msg) => {
                self.cmd = Some("subscribe".into());
                self.params = Some(submsg);
            },
            _ => {}
        }
        self
    }

    /// Construct a KalshiClientMessage from self's current state
    pub fn build(&mut self) -> KalshiClientMessage {
        let message = KalshiClientMessage {
            id: self.next_id,
            cmd: self.cmd.clone().unwrap_or_default(),
            params: self.params.take().unwrap(),
        };
        self.next_id += 1;
        message
    }
}

/// 
/// A sub-message of a KalshiClientMessage.
/// Kalshi messages are structured so that each type of message has 
/// identical structure but for what is under the 'params' field.
/// 
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum KalshiClientSubMessage {
    SubscribeSubMessage(SubscribeSubMessage)
}

/// A sub-message representing a subscription request.
#[derive(Serialize, Deserialize)]
pub struct SubscribeSubMessage {
    channels: Vec<String>
}

impl SubscribeSubMessage {

    /// Construct a new subscription message with the default 'fill' channel
    pub fn default() -> SubscribeSubMessage {
        SubscribeSubMessage { 
            channels: vec!["fill".into()]
        }
    }
}

/// A kalshi market data message
#[derive(Serialize, Deserialize)]
pub struct FillMessage {
    #[serde(rename="type")]
    msg_type: String,
    sid: u32,
    seq: u32,
    pub msg: FillSubMessage
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FillSubMessage {
    Fill(Fill)
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    YES,
    NO
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    BUY,
    SELL
}
