use lapin::{Connection, ConnectionProperties};
use anyhow::Result;
use queue_client::producer::Producer;
use queue_client::queue_data::orders::CreateOrderMessage;
use kalshi::{Action, Side, OrderType};

#[tokio::main]
async fn main() -> Result<()> {

    let addr = "amqp://localhost:5672";
    let connection = Connection::connect(addr, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;

    let producer = Producer::<CreateOrderMessage>::new(channel).await?;

    let mock_create_order = CreateOrderMessage {
        action: Action::Buy,
        client_order_id: "miles69".to_string(),
        count: 1,
        side: Side::Yes,
        ticker: "INXD-23DEC29-B4762".to_string(),
        input_type: OrderType::Limit,
        buy_max_cost: None,
        expiration_ts: None,
        no_price: None,
        sell_position_floor: None,
        yes_price: Some(2),
    };

    producer.publish(mock_create_order).await?;

    Ok(())
}