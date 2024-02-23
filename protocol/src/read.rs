use anyhow::Result;
use tokio::net::TcpStream;
use queue_client::queue_data::orders::CreateOrderMessage;
use queue_client::queue_data::cancels::CancelOrderMessage;

use crate::constants::{LOGIN_HEADER, MESSAGE_LENGTH_SIZE, ORDER_HEADER, CANCEL_HEADER};

enum IncomingMessageType {
    Order,
    Cancel,
    Login,
}

enum IncomingMessage {
    Order(CreateOrderMessage),
    Cancel(CancelOrderMessage),
    Login(String)
}

/*
Reads an order from the TcpStream.
Only call when the message header has already been read and the next message is known to be an order.
*/
async fn read_order(input: &TcpStream) -> Result<CreateOrderMessage> {
    todo!()
}

/*
Reads a cancel from the TcpStream.
Only call when the message header has already been read and the next message is known to be an cancel.
*/
async fn read_cancel(input: &TcpStream) -> Result<CancelOrderMessage> {
    todo!()
}

/*
Returns the next IncomingMessage from the TcpStream
*/
pub async fn read_next(input: &TcpStream) -> Result<IncomingMessage> {
    match read_header(input).await? {
        IncomingMessageType::Order => Ok(IncomingMessage::Order(read_order(input).await?)),
        IncomingMessageType::Cancel => Ok(IncomingMessage::Cancel(read_cancel(input).await?)),
        IncomingMessageType::Login => Ok(IncomingMessage::Login(read_login(input).await?)),
    }
}

pub async fn read_login(input: &TcpStream) -> Result<String> {
    let name_length = read_message_length(input).await?;
    Ok(String::from_utf8(read_n_bytes(input, name_length).await?)?)
}

async fn read_header(input: &TcpStream) -> Result<IncomingMessageType> {
    match read_header_byte(input).await? {
        LOGIN_HEADER => Ok(IncomingMessageType::Login),
        ORDER_HEADER => Ok(IncomingMessageType::Order),
        CANCEL_HEADER => Ok(IncomingMessageType::Cancel),
        _ => Err(anyhow::anyhow!("Invalid header byte")),
    }
}

async fn read_header_byte(input: &TcpStream) -> Result<u8> {
    todo!()
}

async fn read_message_length(input: &TcpStream) -> Result<usize> {
    match read_n_bytes(input, MESSAGE_LENGTH_SIZE).await {
        Ok(bytes) => Ok(bytes[0] as usize),
        Err(e) => Err(e)
    }
}

async fn read_n_bytes(input: &TcpStream, n: usize) -> Result<Vec<u8>> {
    todo!()
}