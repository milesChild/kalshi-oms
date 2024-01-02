use anyhow::Result;
use lapin::{Connection, ConnectionProperties};
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
use tracing::{debug, error, info, warn};

mod constants;

#[tokio::main]
async fn main() -> Result<()> {

    let addr = "amqp://localhost:5672";
    let connection = Connection::connect(addr, ConnectionProperties::default()).await?;

    let order_channel = connection.create_channel().await?;
    let cancel_channel = connection.create_channel().await?;
    
    // problem: how to give the consumer tasks their channels or connections
    // options:
    // 1: construct the consumers here and pass them (more arguments than ideal)
    // 2: pass a connection handle and allow each task to make a channel and a producer (problem: dont want each task locking the connection bc it will always be in scope)
    // 3: force each task to connect separately (seems ridiculous)
    
    let order_producer_handle = Arc::new(Mutex::new(Producer::<CreateOrderMessage>::new(order_channel).await?));
    let cancel_producer_handle = Arc::new(Mutex::new(Producer::<CancelOrderMessage>::new(cancel_channel).await?));

    let client_map_handle = Arc::new(BlockingMutex::new(HashMap::<String, Arc<Mutex<TcpStream>>>::new()));

    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind");
    tokio::spawn(handle_incoming_connections(
        listener, 
        Arc::clone(&client_map_handle), 
        Arc::clone(&order_producer_handle), 
        Arc::clone(&cancel_producer_handle)));

    tokio::spawn(wait_for_order_confirms(client_map_handle.clone()));


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
        let mut name_buffer: [u8; constants::CLIENT_NAME_SIZE_BYTES] = [0; constants::CLIENT_NAME_SIZE_BYTES];
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
    connection_handle: Arc<Mutex<Connection>>
) -> Result<()> {

    todo!()
}

async fn wait_for_order_confirms(clients: Arc<BlockingMutex<HashMap<String, Arc<Mutex<TcpStream>>>>>) -> Result<()> {
    let connection = Connection::connect(constants::MQ_ADDR, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;

    let order_confirm_consumer = Consumer::<OrderConfirmMessage>::new(channel).await?;

    loop {
        let next_confirm = match order_confirm_consumer.get_next().await? {
            None => continue,
            Some(confirm) => confirm
        };

        let (client_id, client_order_id) = match next_confirm.client_order_id {
            None => {
                warn!("Received order confirmation message with no client_order_id. Cannot route to destination client!");
                continue;
            },
            Some(clordid) => {
                match split_client_name(&clordid) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Could not split client_order_id {:?} from OrderConfirmMessag. Cannot route to destination client.", clordid);
                        continue;
                    }
                }
            }
        };
    }

    todo!()
}

async fn wait_for_fills(clients: Arc<BlockingMutex<HashMap<String, Arc<Mutex<TcpStream>>>>>) -> Result<()> {
    

    todo!()
}

fn split_client_name(client_order_id: &str) -> Result<(String, String)> {

    todo!();
}
