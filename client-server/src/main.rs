use anyhow::Result;
use lapin::{
    options::*,
    types::{FieldTable, AMQPValue},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use std::sync::{Arc, Mutex as BlockingMutex};
use std::collections::HashMap;

use queue_client::producer::Producer;
use queue_client::consumer::Consumer;
use queue_client::queue_data::{
    orders::CreateOrderMessage, 
    cancels::CancelOrderMessage, 
    cancels::CancelConfirmMessage, 
    orders::OrderConfirmMessage, 
    fills::FillMessage
};

#[tokio::main]
async fn main() -> Result<()> {

    let addr = "amqp://localhost:5672";
    let connection = Connection::connect(addr, ConnectionProperties::default()).await?;

    let order_channel = connection.create_channel().await?;
    let order_confirm_channel = connection.create_channel().await?;
    let cancel_channel = connection.create_channel().await?;
    let cancel_confirm_channel = connection.create_channel().await?;
    let fill_channel = connection.create_channel().await?;
    
    let order_producer_handle = Arc::new(Mutex::new(Producer::<CreateOrderMessage>::new(order_channel).await?));
    let cancel_producer_handle = Arc::new(Mutex::new(Producer::<CancelOrderMessage>::new(cancel_channel).await?));

    let fill_consumer_handle = Arc::new(Mutex::new(Consumer::<FillMessage>::new(fill_channel).await?));
    let order_confirm_consumer_handle = Arc::new(Mutex::new(Consumer::<OrderConfirmMessage>::new(order_confirm_channel).await?));
    let cancel_confirm_consumer_handle = Arc::new(Mutex::new(Consumer::<CancelConfirmMessage>::new(cancel_confirm_channel).await?));

    let client_map_handle = Arc::new(BlockingMutex::new(HashMap::<String, TcpStream>::new()));

    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind");
    tokio::spawn(handle_incoming_connections(
        listener, 
        Arc::clone(&client_map_handle), 
        Arc::clone(&order_producer_handle), 
        Arc::clone(&cancel_producer_handle)));

    // spawn tasks that listen to fill, order confirm, and cancel confirm channels, determine correct client, and send to client


    Ok(())
}

async fn handle_incoming_connections(
    listener: TcpListener, 
    clients: Arc<BlockingMutex<HashMap<String, TcpStream>>>, 
    order_handle: Arc<Mutex<Producer<CreateOrderMessage>>>,
    cancel_handle: Arc<Mutex<Producer<CancelOrderMessage>>>
) -> Result<()> {

    // while loop to accept clients
        // when accepted, unlock client map and add them
        // then spawn a task with another function that solely listens to the client and queues up orders and cancels

    Ok(())
}
