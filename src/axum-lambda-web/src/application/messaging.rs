use crate::application::events::MessageType;
use async_trait::async_trait;
use aws_sdk_eventbridge::types::PutEventsRequestEntry;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
struct MessageWrapper<T>
where
    T: Serialize,
{
    metadata: Metadata,
    data: T,
}

impl MessageWrapper<MessageType> {
    fn new(message: MessageType) -> Self {
        Self {
            metadata: Metadata::new(&message),
            data: message,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Metadata {
    event_id: String,
    event_date: i64,
    event_type: String,
    event_version: String,
}

impl Metadata {
    fn new(message_type: &MessageType) -> Self {
        let (event_type, event_version) = match &message_type {
            MessageType::ToDoCreated(_) => ("ToDoCreated", "v1"),
            MessageType::ToDoUpdated(_) => ("ToDoUpdated", "v1"),
            MessageType::ToDoCompleted(_) => ("ToDoCompleted", "v1"),
        };

        Self {
            event_id: Uuid::new_v4().to_string(),
            event_date: Utc::now().timestamp(),
            event_type: String::from(event_type),
            event_version: String::from(event_version),
        }
    }
}

#[async_trait]
pub trait MessagePublisher {
    async fn publish(&self, message: MessageType) -> Result<(), ()>;
}

pub struct InMemoryMessagePublisher {}

impl InMemoryMessagePublisher {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MessagePublisher for InMemoryMessagePublisher {
    async fn publish(&self, message: MessageType) -> Result<(), ()> {
        let wrapped_message = MessageWrapper::new(message);

        tracing::info!("{}", serde_json::json!(wrapped_message));

        Ok(())
    }
}

pub struct EventBridgeEventPublisher {
    client: aws_sdk_eventbridge::Client,
}

impl EventBridgeEventPublisher {
    pub fn new(client: aws_sdk_eventbridge::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl MessagePublisher for EventBridgeEventPublisher {
    async fn publish(&self, message: MessageType) -> Result<(), ()> {
        let wrapped_message = MessageWrapper::new(message);

        tracing::info!("{}", serde_json::json!(wrapped_message));

        let publish_res = self
            .client
            .put_events()
            .entries(
                PutEventsRequestEntry::builder()
                    .event_bus_name(env::var("EVENT_BUS_NAME").unwrap())
                    .detail(serde_json::json!(wrapped_message).to_string())
                    .detail_type(&wrapped_message.metadata.event_type)
                    .build(),
            )
            .send()
            .await;

        match publish_res {
            Ok(res) => {
                tracing::info!("{} failed events", res.failed_entry_count);
                Ok(())
            }
            Err(err) => panic!("{}", err.into_service_error()),
        }
    }
}
