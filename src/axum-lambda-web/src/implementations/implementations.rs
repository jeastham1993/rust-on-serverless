use std::fmt::Error;
use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use crate::application::application::{Todo, ToDoRepo};

pub struct DynamoDbToDoRepo {
    client: Client,
    table_name: String
}

impl DynamoDbToDoRepo {
    pub fn new (client: Client, table_name: String) -> Self{
        Self {
            client,
            table_name
        }
    }
}

#[async_trait]
impl ToDoRepo for DynamoDbToDoRepo {
    async fn list(&self, user_id: &str) -> Result<Vec<Todo>, Error> {
        let res = self.client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :hashKey")
            .expression_attribute_values(
                ":hashKey",
                AttributeValue::S(String::from("USER#JAMESEASTHAM")),
            )
            .send()
            .await;

        let query_result = res.unwrap();

        let mut items: Vec<Todo> = Vec::new();

        query_result
            .items()
            .into_iter()
            .for_each(|item| {
                items.push(Todo {
                    id: item["id"].as_s().unwrap().to_string(),
                    text: item["text"].as_s().unwrap().to_string(),
                    completed: *item["completed"].as_bool().unwrap(),
                });
            });

        Ok(items)
    }

    async fn create(&self, todo: Todo) -> Result<(), Error> {
        let _ = self.client
            .put_item()
            .table_name(&self.table_name)
            .item("PK", AttributeValue::S(String::from("USER#JAMESEASTHAM")))
            .item(
                "SK",
                AttributeValue::S(String::from(format!("TODO#{0}", &todo.id.to_uppercase()))),
            )
            .item("text", AttributeValue::S(todo.text.to_string()))
            .item("id", AttributeValue::S(todo.id.to_string()))
            .item("completed", AttributeValue::Bool(todo.completed))
            .send()
            .await;

        Ok(())
    }

    async fn get(&self, todo_id: &str) -> Result<Todo, Error> {
        let res = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S("USER#JAMESEASTHAM".to_string()))
            .key(
                "SK",
                AttributeValue::S(String::from(format!("TODO#{0}", todo_id.to_uppercase()))),
            )
            .send()
            .await;

        let response_value = res.unwrap();
        let result_item = response_value.item().expect("Item should exist");

        Ok(Todo {
            id: result_item["id"].as_s().unwrap().to_string(),
            text: result_item["text"].as_s().unwrap().to_string(),
            completed: *result_item["completed"].as_bool().unwrap(),
        })
    }
}

pub struct InMemoryToDoRepo{
    todos: Vec<Todo>
}

impl InMemoryToDoRepo {
    pub fn new() -> Self{
        Self {
            todos: Vec::new()
        }
    }
}

#[async_trait]
impl ToDoRepo for InMemoryToDoRepo {

    async fn list(&self, user_id: &str) -> Result<Vec<Todo>, Error> {
        Ok(self.todos.clone())
    }

    async fn create(&self, to_do: Todo) -> Result<(), Error> {
        Ok(())
    }

    async fn get(&self, todo_id: &str) -> Result<Todo, Error> {
        let todo: Vec<Todo> = self.todos.clone()
            .into_iter()
            .filter(|todo| todo.id == todo_id)
            .collect();

        if todo.iter().count() == 1 {
            Ok(todo[0].clone())
        }
        else {
            Err(Error)
        }
    }
}