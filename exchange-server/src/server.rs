#[allow(unused_imports)]

extern crate websocket;
use native_tls::TlsStream;
use websocket::sync::Client;
use websocket::{ClientBuilder, OwnedMessage, Message};
use websocket::header::{Headers, Authorization, Bearer};
use std::net::TcpStream;
use std::env;
use log::{debug, info, trace};
use anyhow::Result;
use lapin::{
    options::*,
    types::{FieldTable, AMQPValue},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use queue_client::consumer::Consumer;
use queue_client::producer::Producer;
use queue_client::queue_data::{QueueClass, FillMessage};

use crate::kalshi_wss::SubscribeSubMessage;
use crate::kalshi_wss::KalshiClientSubMessage as SubMessage;

use crate::exchange_client_utils::CreateOrderMessage;
use crate::exchange_client_utils::CancelOrderMessage;

use kalshi::Kalshi;
use kalshi::Order;
use kalshi::OrderType;

// FOR TESTING ONLY
use kalshi::OrderStatus;
use kalshi::Action;
use kalshi::Side;

mod constants;
mod kalshi_wss;
mod exchange_client_utils;

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
    let addr = "amqp://localhost:5672";
    let connection = Connection::connect(addr, ConnectionProperties::default()).await?;
    let producer_channel = connection.create_channel().await?;
    let consumer_channel = connection.create_channel().await?;

    let order_consumer = Consumer::<kalshi::Order>::new(consumer_channel).await?;
    let fill_producer = Producer::<FillMessage>::new(producer_channel).await?;

    // 4. Loop
        
    run_loop(exchange_client, wss_client, order_consumer, fill_producer).await?;

    Ok(())

}

async fn run_loop(mut exchange_client: Kalshi<'_>, mut wss_client: Client<TlsStream<TcpStream>>, mut order_consumer: Consumer<Order>, mut fill_producer: Producer<FillMessage>) -> Result<()> {

    loop {
        // 1. Empty the orders & cancels queue

        let orders: Vec<Order> = order_consumer.get_all().await?;
        // let cancels: Vec<CancelOrderMessage> = Vec::new();

        // 2. Pass orders and cancels to the exchange client

        for order in orders {
            // for each order, unpack the CreateOrderMessage and call the exchange client's create_order method
            debug!("Sending order: {:?}", order);
            let order_response: Order = exchange_client.create_order(order.action,
                Some(order.client_order_id),
                order.place_count.unwrap(),
                order.side,
                order.ticker,
                OrderType::Limit,
                None,
                None,
                None,
                None,
                Some(order.yes_price.into()),
            ).await?;
            debug!("Order Response: {:?}", order_response);
            // match exchange_client.create_order(order).await {
            //     Ok(order) => {
            //         debug!("Successfully created order: {:?}", order);
            //         // submit this order to the "order_confirm" queue using the producer
            //         order_confirm_producer.publish(order).await?;
            //     },
            //     Err(e) => debug!("Failed to create order with error: {:?}", e)
            // }
        }

        // for cancel in cancels {
        //     debug!("Sending cancel: {:?}", cancel);
        // }

        // Make a dummy order
        // let fake_order = Order {
        //     order_id: "O123456".to_string(),
        //     user_id: Some("U123".to_string()),
        //     ticker: "AAPL".to_string(),
        //     status: OrderStatus::Resting,
        //     yes_price: 100,
        //     no_price: 50,
        //     created_time: Some("2023-03-15T12:00:00Z".to_string()),
        //     taker_fill_count: Some(3),
        //     taker_fill_cost: Some(150),
        //     place_count: Some(1),
        //     decrease_count: None,
        //     maker_fill_count: Some(2),
        //     fcc_cancel_count: None,
        //     close_cancel_count: None,
        //     remaining_count: Some(10),
        //     queue_position: Some(5),
        //     expiration_time: Some("2023-03-20T12:00:00Z".to_string()),
        //     taker_fees: Some(5),
        //     action: Action::Buy,
        //     side: Side::Yes,
        //     r#type: "Limit".to_string(),
        //     last_update_time: Some("2023-03-15T12:05:00Z".to_string()),
        //     client_order_id: "C123456".to_string(),
        //     order_group_id: "G123".to_string(),
        // };

        // submit this dummy order to the "orders" queue using the producer
        // debug!("Sending fake order: {:?}", fake_order);
        // mq_producer.publish(fake_order).await?;

        // check if the order was successfully placed in the queue
        // match mq_consumer.get_next().await? {
        //     Some(order) => debug!("Successfully pulled order from the queue: {:?}", order),
        //     None => debug!("Failed to pull order from the queue")
        // }

        // 3. Relay fills from the websocket client to the message queue wrapper

        match wss_client.recv_message().unwrap() {
            OwnedMessage::Text(s) => {
                debug!("Handling incoming text");
                debug!("{s}");
                handle_websocket_text(s, &fill_producer).await?;
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
async fn handle_websocket_text(text: String, producer: &Producer<FillMessage>) -> Result<(), anyhow::Error> {
    
    let wrapper_msg = match serde_json::from_str::<FillMessage>(&text) {
        Ok(msg) => producer.publish(msg).await?,
        Err(_e) => {
            debug!("Ignoring non-FillMessage text data.");
            return Ok(())
        }
    };
    Ok(())
}