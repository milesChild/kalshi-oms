#[allow(unused_imports)]

extern crate websocket;
use log::{debug, info, error};
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
        
        // get all orders in the queue
        match order_consumer.get_all().await {

            Ok(orders) => {
                
                // iteratively place each order & relay response to MQ
                for order in orders {
                    info!("Relaying Order from MQ to Exchange: {:?}", order);

                    match exchange_client.create_order(
                        order.action,
                        Some(order.client_order_id),
                        order.count,
                        order.side,
                        order.ticker,
                        order.input_type,
                        order.buy_max_cost,
                        order.expiration_ts,
                        order.no_price,
                        order.sell_position_floor,
                        order.yes_price,
                    ).await {
                        Ok(order_response) => {
                            debug!("Exchange Order Response: {:?}", order_response);
                            // send the order confirmation to the "order_confirm" queue using the producer
                            let order_confirm = OrderConfirmMessage::new(order_response.order_id, Some(order_response.client_order_id));
                            debug!("Relaying Order Confirmation to MQ: {:?}", order_confirm);
                            order_confirm_producer.publish(order_confirm).await?;
                        },
                        Err(e) => error!("Error placing order: {:?}", e)
                    }
                }
            },
            Err(e) => error!("Error getting orders from queue: {:?}", e)
        }
    }

}