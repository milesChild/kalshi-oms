#[allow(unused_imports)]

extern crate websocket;
use native_tls::TlsStream;
use websocket::sync::Client;
use websocket::{ClientBuilder, OwnedMessage, Message};
use websocket::header::{Headers, Authorization, Bearer};
use std::net::TcpStream;
use std::env;
use log::{debug, info, trace};
use anyhow::anyhow;

use crate::kalshi_wss::SubscribeSubMessage;
use crate::kalshi_wss::KalshiClientSubMessage as SubMessage;
use crate::kalshi_wss::FillMessage;

use crate::exchange_client_utils::CreateOrderMessage;
use crate::exchange_client_utils::CancelOrderMessage;

use kalshi::Kalshi;
use kalshi::Order;

mod constants;
mod kalshi_wss;

extern crate kalshi;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    
    // Initialize logger

    env_logger::Builder::from_default_env().format_timestamp_micros().init();

    // 1. Create and login to a new Kalshi REST API Client

    let (username, password) = (constants::USER, constants::PW);

    let mut exchange_client = Kalshi::new(kalshi::TradingEnvironment::LiveMarketMode);

    exchange_client.login(username, password).await.expect("Could not login to Kalshi.");

    info!("Successful instantiation of kalshi exchange client");

    let token = exchange_client.get_user_token().expect("Could not get user token.");

    info!("Retrieved exchange token: {}", token);

    // 2. Create a new websocket client and subscribe to fills

    let mut custom_headers = Headers::new();
    custom_headers.set(Authorization(token.to_owned()));

    let mut wss_client = ClientBuilder::new(constants::PROD_WSS)
        .unwrap()
        .custom_headers(&custom_headers)
        .connect_secure(None) // Connect with TLS
        .unwrap();

    let mut msg_builder = kalshi_wss::KalshiClientMessageBuilder::new();

    let fills_sub_msg = SubscribeSubMessage::default();
    let init_sub_msg = msg_builder.content(SubMessage::SubscribeSubMessage(fills_sub_msg))
        .build();
    info!("Sending initial fill subscription message: {:?}", serde_json::to_string(&init_sub_msg).unwrap());
    wss_client.send_message(&init_sub_msg.to_websocket_message())?;

    // 3. Create a new message queue wrapper

    // 4. Loop
        
    run_loop(exchange_client, wss_client)

}

fn run_loop(mut exchange_client: Kalshi, mut wss_client: Client<TlsStream<TcpStream>>) -> Result<(), anyhow::Error> {

    loop {
        // 1. Empty the orders & cancels queue

        let orders: Vec<CreateOrderMessage> = Vec::new();
        let cancels: Vec<CancelOrderMessage> = Vec::new();

        // 2. Pass orders and cancels to the exchange client

        for order in orders {
            // for each order, unpack the CreateOrderMessage and call the exchange client's create_order method
            debug!("Sending order: {:?}", order);
            let order: Order = exchange_client.create_order(
                order.action,
                order.client_order_id,
                order.count,
                order.side,
                order.ticker,
                order.input_type,
                order.buy_max_cost,
                order.expiration_ts,
                order.no_price,
                order.sell_position_floor,
                order.yes_price,
            )
            .unwrap();
        }

        for cancel in cancels {
            debug!("Sending cancel: {:?}", cancel);
            let order: Order = exchange_client.cancel_order(cancel.order_id).unwrap();
            info!("Cancelled order: {:?}", order);
        }

        // 3. Relay fills from the websocket client to the message queue wrapper

        match wss_client.recv_message().unwrap() {
            OwnedMessage::Text(s) => {
                debug!("Handling incoming text");
                debug!("{s}");
                handle_websocket_text(s)?
            },
            OwnedMessage::Binary(_b) => debug!("Received and ignored binary data."),
            OwnedMessage::Close(close_data) => {
                info!("Websocket closed by server for reason: {}", close_data.unwrap().reason);
                break;
            },
            OwnedMessage::Ping(data) => match wss_client.send_message(&Message::pong(data)) {
                Ok(()) => trace!("Sent pong in response to ping"),
                Err(e) => panic!("Failed to send pong with error {e:?}") 
            },
            OwnedMessage::Pong(_data) => {} // as a client, we do not expect to receive pongs
        }
    }
    Ok(())
}

// Handle websocket text messages
// fn handle_websocket_text(text: String, mq_wrapper: &mut MessageQueueWrapper) -> Result<(), anyhow::Error> {
fn handle_websocket_text(text: String) -> Result<(), anyhow::Error> {
    
    let wrapper_msg = match serde_json::from_str::<FillMessage>(&text) {
        Ok(msg) => msg,
        Err(_e) => {
            debug!("Ignoring non-FillMessage text data.");
            return Ok(())
        }
    };
    Ok(())
}