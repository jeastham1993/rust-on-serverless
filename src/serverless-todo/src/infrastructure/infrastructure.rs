use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};

use crate::domain::entities::{ToDo, OwnerId, Title};
use crate::domain::{
    entities::{Repository, ToDoId},
    error_types::RepositoryError
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
    async fn store_todo(&self, body: &ToDo) -> Result<(), RepositoryError> {
        tracing::info!("Storing record in DynamoDB");

        let res = self
            .client
            .put_item()
            .table_name(self.table_name)
            .item("PK", AttributeValue::S(body.get_owner().to_string()))
            .item("SK", AttributeValue::S(body.get_id().to_string()))
            .item(
                "id",
                AttributeValue::S(body.get_id().into()),
            )
            .item(
                "title",
                AttributeValue::S(body.get_title().into()),
            )
            .item(
                "status",
                AttributeValue::S(body.get_status().into()),
            )
            .item(
                "ownerId",
                AttributeValue::S(body.get_owner().into()),
            )
            .send()
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(RepositoryError::new(e.to_string())),
        }
    }

    async fn get_todo(&self, owner: &String, id: &String) -> Result<ToDo, RepositoryError> {
        tracing::info!("Retrieving record from DynamoDB");

        let res = self
            .client
            .get_item()
            .table_name(self.table_name)
            .key("PK", AttributeValue::S(owner.to_string()))
            .key("SK", AttributeValue::S(id.to_string()))
            .send()
            .await;

        match res {
            Ok(_) => Ok({
                let item = res.unwrap();
                let attributes = item.item().unwrap().clone();

                ToDo::new(Title::Title(attributes.get("title").unwrap().as_s().unwrap().clone()),
                    OwnerId::OwnerId(attributes.get("ownerId").unwrap().as_s().unwrap().clone()),
                    Some(attributes.get("status").unwrap().as_s().unwrap().clone()),
                    Some(ToDoId::ToDoId(attributes.get("id").unwrap().as_s().unwrap().clone()))).unwrap()
            }),
            Err(e) => Err(RepositoryError::new(e.to_string())),
        }
    }

    async fn list_todos(&self, owner: &String) -> Result<Vec<ToDo>, RepositoryError> {
        tracing::info!("Retrieving record from DynamoDB");

        let res = self
            .client
            .query()
            .table_name(self.table_name)
            .key_condition_expression(
                "PK = :hashKey",
            )
            .expression_attribute_values(
                ":hashKey",
                AttributeValue::S(owner.to_string()),
            )
            .send()
            .await;

        match res {
            Ok(query_res) => Ok({

                let mut items: Vec<ToDo> = Vec::new();

                for item in query_res.items().unwrap() {
                    items.push(
                        ToDo::new(Title::Title(item.get("title").unwrap().as_s().unwrap().clone()),
                        OwnerId::OwnerId(item.get("ownerId").unwrap().as_s().unwrap().clone()),
                        Some(item.get("status").unwrap().as_s().unwrap().clone()),
                        Some(ToDoId::ToDoId(item.get("id").unwrap().as_s().unwrap().clone()))).unwrap())
                }

                items
            }),
            Err(e) => Err(RepositoryError::new(e.to_string())),
        }
    }
}
