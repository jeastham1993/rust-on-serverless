use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, model::AttributeValue};

use crate::domain::domain::{Repository, RepositoryError};

use crate::domain::entities::{ToDoItem, ToDo};

pub struct DynamoDbRepository<'a> {
    client: Client,
    table_name: &'a String,
}

impl DynamoDbRepository<'_> {
    pub fn new(client: Client, table_name: &String) -> DynamoDbRepository{
        return DynamoDbRepository { client: client, table_name: table_name }
    }
}

#[async_trait]
impl Repository for DynamoDbRepository<'_> {
    async fn store_todo(&self, body: &ToDo) -> Result<String, RepositoryError> {

        tracing::info!("Storing record in DynamoDB");

        let res = self.client
            .put_item()
            .table_name(self.table_name)
            .item("id", AttributeValue::S(body.id.value().to_string()))
            .item("title", AttributeValue::S(body.title.value().to_string()))
            .item("isComplete", AttributeValue::S(body.is_complete.to_string()))
            .item("ownerId", AttributeValue::S(body.owner_id.value().to_string()))
            .send()
            .await;

        match res {
            Ok(_) => Ok("OK".to_string()),
            Err(e) => Err(RepositoryError::new(e.to_string()))
        }
    }
    
    async fn get_todo(&self, id: &String) -> Result<ToDoItem, RepositoryError> {

        tracing::info!("Retrieving record from DynamoDB");

        let res = self.client
            .get_item()
            .table_name(self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await;

        match res {
            Ok(_) => Ok({
                let item = res.unwrap();
                let attributes = item.item().unwrap().clone();

                ToDoItem {
                    id: attributes.get("id").unwrap().as_s().unwrap().clone(),
                    title: attributes.get("title").unwrap().as_s().unwrap().clone(),
                    is_complete: attributes.get("isComplete").unwrap().as_s().unwrap().clone()
                }
            }),
            Err(e) => Err(RepositoryError::new(e.to_string()))
        }
    }
}