use std::marker::PhantomData;
use lapin::{
    options::*,
    types::FieldTable,
    Channel
};
use anyhow::Result;

use crate::queue_data::QueueData;

// Your struct representing the producer
pub struct Producer<T: QueueData> {
    channel: Channel,
    queue_name: String,
    phantom_data: PhantomData<T>
}

impl<T: QueueData> Producer<T> {
    // Create a new producer
    pub async fn new(channel: Channel) -> Result<Self> {
        //let queue_name = T::class().to_string();
        let queue_name = T::class().to_string();
        // Declare the exchange
        channel
            .queue_declare(&queue_name, QueueDeclareOptions::default(), FieldTable::default())
            .await?;

        Ok(Producer {
            channel,
            queue_name: queue_name,
            phantom_data: PhantomData
        })
    }

    // Publish a single element to the queue
    pub async fn publish(&self, message: T) -> Result<()> {
        let serialized_data = message.serialize()?;
        print!("{:?}", serialized_data);
        self.channel.basic_publish(
            "",
            &self.queue_name,
            BasicPublishOptions::default(),
            serialized_data.as_slice(),
            Default::default(),
        ).await?;
        Ok(())
    }

    // Publish a vector of elements to the queue
    pub async fn publish_batch(&self, messages: Vec<T>) -> Result<()> {
        for message in messages {
            self.publish(message).await?;
        }
        Ok(())
    }
}