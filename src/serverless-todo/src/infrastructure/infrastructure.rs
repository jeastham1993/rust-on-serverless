use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use uuid::Uuid;

use crate::domain::public_types::ValidatedToDo;
use crate::domain::{
    entities::{Repository, ToDoId},
    error_types::RepositoryError,
    public_types::{CreatedToDo, ToDoItem},
};

pub struct DynamoDbRepository<'a> {
    client: Client,
    table_name: &'a String,
}

impl DynamoDbRepository<'_> {
    pub fn new(client: Client, table_name: &String) -> DynamoDbRepository {
        return DynamoDbRepository {
            client: client,
            table_name: table_name,
        };
    }
}

#[async_trait]
impl Repository for DynamoDbRepository<'_> {
    async fn store_todo(&self, body: ValidatedToDo) -> Result<CreatedToDo, RepositoryError> {
        tracing::info!("Storing record in DynamoDB");

        let created_to_do = CreatedToDo {
            title: body.title,
            id: ToDoId::new(Uuid::new_v4().to_string()).unwrap(),
            is_complete: body.is_complete,
            owner_id: body.owner_id,
        };

        let res = self
            .client
            .put_item()
            .table_name(self.table_name)
            .item(
                "id",
                AttributeValue::S(created_to_do.id.get_value()),
            )
            .item(
                "title",
                AttributeValue::S(created_to_do.title.get_value()),
            )
            .item(
                "isComplete",
                AttributeValue::S(created_to_do.is_complete.to_string()),
            )
            .item(
                "ownerId",
                AttributeValue::S(created_to_do.owner_id.get_value()),
            )
            .send()
            .await;

        match res {
            Ok(_) => Ok(created_to_do),
            Err(e) => Err(RepositoryError::new(e.to_string())),
        }
    }

    async fn get_todo(&self, id: &String) -> Result<ToDoItem, RepositoryError> {
        tracing::info!("Retrieving record from DynamoDB");

        let res = self
            .client
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
                    is_complete: attributes
                        .get("isComplete")
                        .unwrap()
                        .as_s()
                        .unwrap()
                        .clone(),
                }
            }),
            Err(e) => Err(RepositoryError::new(e.to_string())),
        }
    }
}
