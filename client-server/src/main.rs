use anyhow::Result;
use lapin::{
    options::*,
    types::{FieldTable, AMQPValue},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::io::AsyncReadExt;
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

    let client_map_handle = Arc::new(BlockingMutex::new(HashMap::<String, Arc<Mutex<TcpStream>>>::new()));

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
    clients: Arc<BlockingMutex<HashMap<String, Arc<Mutex<TcpStream>>>>>, 
    order_handle: Arc<Mutex<Producer<CreateOrderMessage>>>,
    cancel_handle: Arc<Mutex<Producer<CancelOrderMessage>>>
) -> Result<()> {
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut name_buffer: [u8; 10] = [0; 10];
        let name_length = socket.read(&mut name_buffer).await?;
        
        if let Ok(name_str) = core::str::from_utf8(&name_buffer[..name_length]) {
            let socket_handle = Arc::new(Mutex::new(socket));
            let name = name_str.to_string();

            { // block off client interaction so the map exits scope and is freed sooner
                let mut clients = clients.lock().unwrap();
                clients.insert(name.clone(), socket_handle.clone());
            }
            
            let order_handle = order_handle.clone();
            let cancel_handle = cancel_handle.clone();
            tokio::spawn(handle_client(socket_handle.clone(), name.clone(), order_handle, cancel_handle));
        }
    }
}

async fn handle_client(
    socket_handle: Arc<Mutex<TcpStream>>,
    name: String,
    order_handle: Arc<Mutex<Producer<CreateOrderMessage>>>,
    cancel_handle: Arc<Mutex<Producer<CancelOrderMessage>>>
) -> Result<()> {

    Ok(())
}

async fn wait_for_cancel_confirms(
    clients: Arc<BlockingMutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    cancel_confirm_handle: Arc<Mutex<Consumer<CancelConfirmMessage>>>
) -> Result<()> {

    todo!()
}

async fn wait_for_order_confirms(
    clients: Arc<BlockingMutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    order_confirm_handle: Arc<Mutex<Consumer<OrderConfirmMessage>>>
) -> Result<()> {

    todo!()
}

async fn wait_for_fills(
    clients: Arc<BlockingMutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    fill_handle: Arc<Mutex<Consumer<FillMessage>>>
) -> Result<()> {

    todo!()
}
