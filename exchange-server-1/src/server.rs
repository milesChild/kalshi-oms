#[allow(unused_imports)]

extern crate websocket;
use log::{debug, info};
use anyhow::Result;
use lapin::{Connection, ConnectionProperties};
use queue_client::{consumer::Consumer, queue_data::orders::OrderConfirmMessage, queue_data::orders::CreateOrderMessage};
use queue_client::producer::Producer;

use kalshi::Kalshi;
use kalshi::Order;

mod constants;

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
    let order_confirm_producer = Producer::<OrderConfirmMessage>::new(producer_channel).await?;

    // 4. Loop
        
    run_loop(exchange_client, order_consumer, order_confirm_producer).await?;

    Ok(())

}

async fn run_loop(exchange_client: Kalshi<'_>, order_consumer: Consumer<CreateOrderMessage>, order_confirm_producer: Producer<OrderConfirmMessage>) -> Result<()> {

    loop {
        // 1. Empty the orders & cancels queue

        //let orders: Vec<CreateOrderMessage> = order_consumer.get_all().await?;
        let orders = match order_consumer.get_all().await {
            Ok(orders) => orders,
            Err(e) => {
                info!("Error getting orders from queue: {:?}", e);
                Vec::new()
            }
        };
        
        // 2. Send orders to exchange

        for order_create in orders {
            // for each order, unpack the CreateOrderMessage and call the exchange client's create_order method
            info!("Relating Order from MQ to Exchange: {:?}", order_create);
            let order_response: Order = exchange_client.create_order(
                order_create.action,
                Some(order_create.client_order_id),
                order_create.count,
                order_create.side,
                order_create.ticker,
                order_create.input_type,
                order_create.buy_max_cost,
                order_create.expiration_ts,
                order_create.no_price,
                order_create.sell_position_floor,
                order_create.yes_price,
            ).await?;
            debug!("Exchange Order Response: {:?}", order_response);
            // send the order confirmation to the "order_confirm" queue using the producer
            let order_confirm = OrderConfirmMessage::new(order_response.order_id, Some(order_response.client_order_id));
            debug!("Sending order confirmation: {:?}", order_confirm);
            order_confirm_producer.publish(order_confirm).await?;
        }

    }

}