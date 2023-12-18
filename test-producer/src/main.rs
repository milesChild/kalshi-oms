use std::sync::{Arc, Mutex};
use lapin::{
    options::*,
    types::{FieldTable, AMQPValue},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use anyhow::Result;
use kalshi::Order;
use queue_client::queue_data::{QueueData, QueueClass};
use queue_client::consumer::Consumer;
use queue_client::producer::Producer;
use kalshi::{Action, Side, OrderStatus};

#[tokio::main]
async fn main() -> Result<()> {

    let addr = "amqp://localhost:5672";
    let connection = Connection::connect(addr, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;

    let producer = Producer::<Order>::new(channel).await?;
    
    let fake_order = Order {
        order_id: "O123456".to_string(),
        user_id: Some("U123".to_string()),
        ticker: "AAPL".to_string(),
        status: OrderStatus::Resting,
        yes_price: 100,
        no_price: 50,
        created_time: Some("2023-03-15T12:00:00Z".to_string()),
        taker_fill_count: Some(3),
        taker_fill_cost: Some(150),
        place_count: Some(1),
        decrease_count: None,
        maker_fill_count: Some(2),
        fcc_cancel_count: None,
        close_cancel_count: None,
        remaining_count: Some(10),
        queue_position: Some(5),
        expiration_time: Some("2023-03-20T12:00:00Z".to_string()),
        taker_fees: Some(5),
        action: Action::Buy,
        side: Side::Yes,
        r#type: "Limit".to_string(),
        last_update_time: Some("2023-03-15T12:05:00Z".to_string()),
        client_order_id: "C123456".to_string(),
        order_group_id: "G123".to_string(),
    };

    producer.publish(fake_order).await?;

    Ok(())
}
