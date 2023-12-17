use std::marker::PhantomData;
use lapin::{
    options::*,
    types::{FieldTable, AMQPValue},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use anyhow::Result;

use crate::queue_data::{QueueData, QueueClass};

pub struct Consumer<T: QueueData> {
    channel: Channel,
    name: String,
    phantom_data: PhantomData<T>,
}

impl<T: QueueData> Consumer<T> {

    async fn new(channel: Channel) -> Result<Self> {
        let name = T::class().to_string();
        // Declare the queue
        channel
            .queue_declare(name.as_str(), QueueDeclareOptions::default(), FieldTable::default())
            .await?;

        Ok(Consumer { channel, name, phantom_data: PhantomData })
    }

    async fn get_next(&self) -> Result<Option<T>> {
        let delivery = self.channel.basic_get(&self.name, BasicGetOptions::default()).await?;
        if let Some(delivery) = delivery {
            Ok(Some(T::deserialize(delivery.data.as_slice())?))
        } else {
            Ok(None)
        }
    }

    async fn get_all(&self) -> Result<Vec<T>> {
        let mut messages = Vec::new();
        loop {
            match self.get_next().await {
                Ok(Some(message)) => messages.push(message),
                Ok(None) => break,
                Err(err) => return Err(err),
            }
        }
        Ok(messages)
    }

}