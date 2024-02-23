use anyhow::Result;
use lapin::{Connection, ConnectionProperties};
use queue_client::queue_data::data_core::QueueData;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, MutexGuard};
use tokio::io::AsyncReadExt;
use std::sync::Arc;
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

    let client_map_handle = Arc::new(Mutex::new(HashMap::<String, Arc<Mutex<TcpStream>>>::new()));

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
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>, 
    order_handle: Arc<Mutex<Producer<CreateOrderMessage>>>,
    cancel_handle: Arc<Mutex<Producer<CancelOrderMessage>>>
) -> Result<()> {
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        
        if let Ok(name) = protocol::read::read_login(&socket).await {
            let socket_handle = Arc::new(Mutex::new(socket));

            { // block off client interaction so the map exits scope and is freed sooner
                let mut clients = clients.lock().await;
                clients.insert(name.clone(), socket_handle.clone());
            }
            
            let order_handle = order_handle.clone();
            let cancel_handle = cancel_handle.clone();
            tokio::spawn(handle_client(socket_handle.clone(), name, order_handle, cancel_handle));
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
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    connection_handle: Arc<Mutex<Connection>>
) -> Result<()> {

    let connection = Connection::connect(constants::MQ_ADDR, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;

    let cancel_confirm_consumer = Consumer::<CancelConfirmMessage>::new(channel).await?;

    loop {
        let next_cancel = match cancel_confirm_consumer.get_next().await? {
            None => continue,
            Some(cancel) => cancel
        };

        let (client_id, client_order_id) = match split_client_name(&next_cancel.client_order_id) {
            Ok(p) => p,
            Err(e) => {
                warn!("Could not split client_order_id {:?} from OrderConfirmMessage. Cannot route to destination client.", &next_cancel.client_order_id);
                continue;
            }
        };

        // NEED TO REPLACE CLORDID IN NEXT_CANCEL WITH THE EXTRACTED ONE
            
        let map_handle = clients.lock().await;
        let client = match map_handle.get(&client_id) {
            None => {
                warn!("No client found corresponding to client id {:?}. Cannot route to destination client.", client_id);
                continue;
            },
            Some(client_handle) => client_handle.lock().await
        };
        write_next_frame(&next_cancel.to_bytes()?, client)?;
    }
}

/// Listen to RabbitMQ for order confirmation messages and route them to the appropriate clients.
async fn wait_for_order_confirms(clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>) -> Result<()> {
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
            Some(ref clordid) => {
                match split_client_name(&clordid) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Could not split client_order_id {:?} from OrderConfirmMessage. Cannot route to destination client.", clordid);
                        continue;
                    }
                }
            }
        };

        let map_handle = clients.lock().await;
        let client = match map_handle.get(&client_id) {
            None => {
                warn!("No client found corresponding to client id {:?}. Cannot route to destination client.", client_id);
                continue;
            },
            Some(client_handle) => client_handle.lock().await
        };
        write_next_frame(&next_confirm.to_bytes()?, client)?;
    }
}

async fn wait_for_fills(clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>) -> Result<()> {
    todo!()
}

/// Deconstruct an exchange-side client order ID into an internal client identifier 
/// and the client's provided order ID.
fn split_client_name(client_order_id: &str) -> Result<(String, String)> {
    todo!();
}

/// Writes the message serialized to bytes, where the leading byte indicates the message length.
fn write_next_frame(bytes: &[u8], stream: MutexGuard<TcpStream>) -> Result<()> {
    todo!();
}

/// Read the next QueueData off of the referenced stream
fn read_next<T: QueueData>(stream: &TcpStream) -> Option<T> {

    todo!();
} 
