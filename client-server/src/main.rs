use anyhow::Result;
use lapin::{
    options::*,
    types::{FieldTable, AMQPValue},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use std::sync::{Arc, Mutex};
use queue_client::producer::Producer;
use queue_client::consumer::Consumer;

#[tokio::main]
async fn main() -> Result<()> {

    let addr = "amqp://localhost:5672";
    let connection = Connection::connect(addr, ConnectionProperties::default()).await?;
    let order_channel_handle = Arc::new(Mutex::new(connection.create_channel().await?));

    Ok(())
}
