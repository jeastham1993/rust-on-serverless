use std::collections::HashMap;
use crate::application::domain::{OwnerId, Title, ToDo, ToDoId, ToDoRepo};
use crate::application::error_types::RepositoryError;
use async_trait::async_trait;
use aws_sdk_dynamodb::error::ProvideErrorMetadata;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use chrono::{DateTime};

pub struct DynamoDbToDoRepo {
    client: Client,
    table_name: String,
}

impl DynamoDbToDoRepo {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }
}

#[async_trait]
impl ToDoRepo for DynamoDbToDoRepo {
    async fn list(&self, user_id: &str) -> Result<Vec<ToDo>, RepositoryError> {
        let res = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :hashKey")
            .expression_attribute_values(":hashKey", generate_pk(user_id))
            .send()
            .await;

        match res {
            Ok(query_res) => Ok({
                let mut items: Vec<ToDo> = Vec::new();

                for item in query_res.items() {
                    items.push(
                        parse_todo_from_item(item)
                    )
                }

                items
            }),
            Err(e) => Err(RepositoryError::new(e.to_string())),
        }
    }

    async fn create(&self, todo: &ToDo) -> Result<(), RepositoryError> {
        let mut dynamo_request_builder = self
            .client
            .put_item()
            .table_name(&self.table_name)
            .item("PK", generate_pk(todo.get_owner()))
            .item("SK", generate_sk(todo.get_id()))
            .item("id", AttributeValue::S(todo.get_id().into()))
            .item("title", AttributeValue::S(todo.get_title().into()))
            .item("status", AttributeValue::S(todo.get_status()))
            .item("ownerId", AttributeValue::S(todo.get_owner().into()));

        if !todo.get_completed_on().is_empty() {
            dynamo_request_builder = dynamo_request_builder.item(
                "completedOn",
                AttributeValue::S(todo.get_completed_on()),
            );
        }

        if !todo.get_description().is_empty() {
            dynamo_request_builder = dynamo_request_builder
                .item("description", AttributeValue::S(todo.get_description().to_string()));
        }

        if !todo.get_due_date().is_empty() {
            dynamo_request_builder = dynamo_request_builder.item(
                "dueDate",
                AttributeValue::S(todo.get_due_date().to_string()),
            );
        }

        let _ = dynamo_request_builder.send().await;

        Ok(())
    }

    async fn get(&self, user_id: &str, todo_id: &str) -> Result<ToDo, RepositoryError> {
        let res = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", generate_pk(user_id))
            .key("SK", generate_sk(todo_id))
            .send()
            .await;

        match res {
            Ok(item) => Ok({
                let attributes = item.item().unwrap().clone();

                parse_todo_from_item(&attributes)
            }),
            Err(e) => Err(RepositoryError::new(
                e.into_service_error().message().unwrap().to_string(),
            )),
        }
    }
}

fn parse_todo_from_item(item: &HashMap<String, AttributeValue>) -> ToDo {
    ToDo::parse(
        Title::new(item.get("title").unwrap().as_s().unwrap()).unwrap(),
        OwnerId::new(item.get("ownerId").unwrap().as_s().unwrap())
            .unwrap(),
        Some(item.get("status").unwrap().as_s().unwrap().clone()),
        Some(
            ToDoId::parse(item.get("id").unwrap().as_s().unwrap())
                .unwrap(),
        ),
        item.get("description").map(|val| val.as_s().unwrap().clone()),
        item.get("dueDate").map(|val| DateTime::parse_from_rfc3339(val.as_s().unwrap()).unwrap()),
        item.get("completedOn").map(|val| DateTime::parse_from_rfc3339(val.as_s().unwrap()).unwrap()),
    )
        .unwrap()
}

fn generate_pk(user_id: &str) -> AttributeValue {
    AttributeValue::S(format!("USER#{0}", user_id.to_uppercase()))
}

fn generate_sk(todo_id: &str) -> AttributeValue {
    AttributeValue::S(format!("TODO#{0}", todo_id.to_uppercase()))
}
