use std::marker::PhantomData;
use lapin::{
    options::*,
    types::FieldTable,
    Channel
};
use anyhow::Result;

use crate::queue_data::QueueData;

pub struct Consumer<'a, T: QueueData> {
    channel: &'a Channel,
    queue_name: String,
    phantom_data: PhantomData<T>,
}

impl<'a, T: QueueData> Consumer<'a, T> {

    pub async fn new(channel: &'a Channel) -> Result<Self> {
        let queue_name = T::class().to_string();
        // Declare the queue
        channel
            .queue_declare(&queue_name, QueueDeclareOptions::default(), FieldTable::default())
            .await?;

        Ok(Consumer { channel, queue_name, phantom_data: PhantomData })
    }

    pub async fn get_next(&self) -> Result<Option<T>> {
        let delivery = self.channel.basic_get(&self.queue_name, BasicGetOptions::default()).await?;
        if let Some(delivery) = delivery {
            Ok(Some(T::from_bytes(&delivery.data)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all(&self) -> Result<Vec<T>> {
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