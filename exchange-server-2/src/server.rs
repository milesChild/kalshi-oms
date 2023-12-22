#[allow(unused_imports)]

extern crate websocket;
use native_tls::TlsStream;
use websocket::sync::Client;
use websocket::{ClientBuilder, OwnedMessage, Message};
use websocket::header::{Headers, Authorization};
use std::net::TcpStream;
use log::{debug, info, trace};
use anyhow::Result;
use lapin::{
    options::*,
    types::{FieldTable, AMQPValue},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use queue_client::producer::Producer;
use queue_client::queue_data::fills::FillMessage;

use crate::kalshi_wss::SubscribeSubMessage;
use crate::kalshi_wss::KalshiClientSubMessage as SubMessage;

use kalshi::Kalshi;

mod constants;
mod kalshi_wss;

extern crate kalshi;

#[tokio::main]
async fn main() -> Result<()> {
    
    // Initialize logger

    env_logger::Builder::from_default_env().format_timestamp_micros().init();

    // 1. Create and login to a new Kalshi REST API Client

    let (username, password) = (constants::USER, constants::PW);

    let mut exchange_client = Kalshi::new(kalshi::TradingEnvironment::LiveMarketMode);

    exchange_client.login(&username, &password).await.expect("Could not login to Kalshi.");

    info!("Successful instantiation of kalshi exchange client");

    let token = exchange_client.get_user_token().expect("Could not get user token.");

    info!("Retrieved exchange token: {}", token);

    // 2. Create a new websocket client and subscribe to fills

    let mut custom_headers = Headers::new();
    custom_headers.set(Authorization(token.to_owned()));

    let mut ws_client = ClientBuilder::new(constants::PROD_WSS)
        .unwrap()
        .custom_headers(&custom_headers)
        .connect_secure(None) // Connect with TLS
        .unwrap();

    let mut msg_builder = kalshi_wss::KalshiClientMessageBuilder::new();

    let fills_sub_msg = SubscribeSubMessage::default();
    let init_sub_msg = msg_builder.content(SubMessage::SubscribeSubMessage(fills_sub_msg))
        .build();
    info!("Sending initial fill subscription message: {:?}", serde_json::to_string(&init_sub_msg).unwrap());
    ws_client.send_message(&init_sub_msg.to_websocket_message())?;

    // 3. Create a new message queue wrapper
    let addr = "amqp://localhost:5672";
    let connection = Connection::connect(addr, ConnectionProperties::default()).await?;
    let producer_channel = connection.create_channel().await?;

    let fill_producer = Producer::<FillMessage>::new(producer_channel).await?;

    // 4. Loop
        
    run_loop(ws_client, fill_producer).await?;

    Ok(())

}

async fn run_loop(mut ws_client: Client<TlsStream<TcpStream>>, fill_producer: Producer<FillMessage>) -> Result<()> {

    loop {
        // 2. Relay fill messages to message queue

        match ws_client.recv_message().unwrap() {
            OwnedMessage::Text(s) => {
                trace!("Handling incoming text");
                trace!("{s}");
                handle_websocket_text(s, &fill_producer).await?;
            },
            OwnedMessage::Binary(_b) => debug!("Received and ignored binary data."),
            OwnedMessage::Close(close_data) => {
                info!("Websocket closed by server for reason: {}", close_data.unwrap().reason);
                break;
            },
            OwnedMessage::Ping(data) => match ws_client.send_message(&Message::pong(data)) {
                Ok(()) => trace!("Sent pong in response to ping"),
                Err(e) => panic!("Failed to send pong with error {e:?}") 
            },
            OwnedMessage::Pong(_data) => {} // as a client, we do not expect to receive pongs
        }
    }
    Ok(())
}

// Handle websocket text messages
async fn handle_websocket_text(text: String, producer: &Producer<FillMessage>) -> Result<(), anyhow::Error> {
    
    let _ = match serde_json::from_str::<FillMessage>(&text) {
        Ok(msg) => producer.publish(msg).await?,
        Err(_e) => {
            debug!("Ignoring non-FillMessage text data.");
            return Ok(())
        }
    };
    Ok(())

    // add match producer.publish(msg).await? with an error here

}