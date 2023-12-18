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

    let consumer = Consumer::<Order>::new(channel).await?;
    
    match consumer.get_next().await? {
        Some(ord) => print!("{:?}", ord),
        None => print!("nothing on queue")
    };

    Ok(())
}
