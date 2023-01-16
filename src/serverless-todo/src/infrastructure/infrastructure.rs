use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use chrono::{DateTime};

use crate::domain::entities::{OwnerId, Title, ToDo};
use crate::domain::{
    entities::{Repository, ToDoId},
    error_types::RepositoryError,
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
            .item("id", AttributeValue::S(body.get_id().into()))
            .item("title", AttributeValue::S(body.get_title().into()))
            .item("status", AttributeValue::S(body.get_status().into()))
            .item("ownerId", AttributeValue::S(body.get_owner().into()))
            .item(
                "completedOn",
                AttributeValue::S(body.get_completed_on().into()),
            )
            .send()
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("{}", e.to_string());

                Err(RepositoryError::new(e.to_string()))
            },
        }
    }

    async fn get_todo(&self, owner: &String, id: &String) -> Result<ToDo, RepositoryError> {
        tracing::info!("Retrieving record from DynamoDB: {owner} {id}");

        let res = self
            .client
            .get_item()
            .table_name(self.table_name)
            .key("PK", AttributeValue::S(owner.to_string()))
            .key("SK", AttributeValue::S(id.to_string()))
            .send()
            .await;

        match res {
            Ok(item) => Ok({
                let attributes = item.item().unwrap().clone();

                ToDo::parse(
                    Title::new(attributes.get("title").unwrap().as_s().unwrap().clone()).unwrap(),
                    OwnerId::new(attributes.get("ownerId").unwrap().as_s().unwrap().clone())
                        .unwrap(),
                    Some(attributes.get("status").unwrap().as_s().unwrap().clone()),
                    Some(
                        ToDoId::parse(attributes.get("id").unwrap().as_s().unwrap().clone())
                            .unwrap(),
                    ),
                    match attributes.get("completedOn") {
                        Option::None => Option::None,
                        Some(val) => {
                            Some(DateTime::parse_from_rfc3339(val.as_s().unwrap()).unwrap())
                        }
                    },
                )
                .unwrap()
            }),
            Err(e) => {
                Err(RepositoryError::new(e.into_service_error().message().unwrap().to_string()))
            },
        }
    }

    async fn list_todos(&self, owner: &String) -> Result<Vec<ToDo>, RepositoryError> {
        tracing::info!("Retrieving record from DynamoDB");

        let res = self
            .client
            .query()
            .table_name(self.table_name)
            .key_condition_expression("PK = :hashKey")
            .expression_attribute_values(":hashKey", AttributeValue::S(owner.to_string()))
            .send()
            .await;

        match res {
            Ok(query_res) => Ok({
                let mut items: Vec<ToDo> = Vec::new();

                for item in query_res.items().unwrap() {
                    items.push(
                        ToDo::parse(
                            Title::new(item.get("title").unwrap().as_s().unwrap().clone()).unwrap(),
                            OwnerId::new(item.get("ownerId").unwrap().as_s().unwrap().clone())
                                .unwrap(),
                            Some(item.get("status").unwrap().as_s().unwrap().clone()),
                            Some(
                                ToDoId::parse(item.get("id").unwrap().as_s().unwrap().clone())
                                    .unwrap(),
                            ),
                            match item.get("completedOn") {
                                Option::None => Option::None,
                                Some(val) => {
                                    Some(DateTime::parse_from_rfc3339(val.as_s().unwrap()).unwrap())
                                }
                            },
                        )
                        .unwrap(),
                    )
                }

                items
            }),
            Err(e) => Err(RepositoryError::new(e.to_string())),
        }
    }
}
