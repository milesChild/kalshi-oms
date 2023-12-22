#[allow(unused_imports)]

extern crate websocket;
use native_tls::TlsStream;
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

    // 3. Create a new message queue wrapper
    let addr = "amqp://localhost:5672";
    let connection = Connection::connect(addr, ConnectionProperties::default()).await?;
    let producer_channel = connection.create_channel().await?;
    let consumer_channel = connection.create_channel().await?;

    let order_consumer = Consumer::<CreateOrderMessage>::new(consumer_channel).await?;
    //let order_producer = Producer::<kalshi::Order>::new(producer_channel).await?;

    // 4. Loop
        
    run_loop(exchange_client, order_consumer).await?;

    Ok(())

}

async fn run_loop(mut exchange_client: Kalshi<'_>, mut order_consumer: Consumer<CreateOrderMessage>) -> Result<()> {

    loop {
        // 1. Empty the orders & cancels queue

        let orders: Vec<Order> = order_consumer.get_all().await?;

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
            
        }

        // submit this dummy order to the "orders" queue using the producer
        // debug!("Sending fake order: {:?}", fake_order);
        // mq_producer.publish(fake_order).await?;

        // check if the order was successfully placed in the queue
        // match mq_consumer.get_next().await? {
        //     Some(order) => debug!("Successfully pulled order from the queue: {:?}", order),
        //     None => debug!("Failed to pull order from the queue")
        // }

    }

    Ok(())
}