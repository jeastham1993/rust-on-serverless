pub mod services {
    use aws_sdk_dynamodb::{model::AttributeValue, Client};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    pub struct TodoService {
        client: Client,
        table_name: String,
    }

    impl TodoService {
        pub fn new(client: Client, table_name: String) -> TodoService {
            TodoService {
                client,
                table_name,
            }
        }

        pub async fn list_todos(&self, username: String) -> Vec<Todo> {
            let res = &self
                .client
                .query()
                .table_name(&self.table_name)
                .key_condition_expression("PK = :hashKey")
                .expression_attribute_values(
                    ":hashKey",
                    AttributeValue::S(format!("USER#{}", username.to_uppercase())),
                )
                .send()
                .await;

            let query_result = res.as_ref().unwrap();

            let mut items: Vec<Todo> = Vec::new();

            query_result
                .items()
                .expect("Items to exist")
                .iter()
                .for_each(|item| {
                    items.push(Todo {
                        id: item["id"].as_s().unwrap().to_string(),
                        text: item["text"].as_s().unwrap().to_string(),
                        completed: *item["completed"].as_bool().unwrap(),
                    });
                });

            items
        }

        pub async fn create_todo(&self, username: String, input: CreateTodo) {
            let todo = Todo {
                completed: false,
                text: input.text.clone(),
                id: Uuid::new_v4().to_string(),
            };

            let _res = &self
                .client
                .put_item()
                .table_name(&self.table_name)
                .item(
                    "PK",
                    AttributeValue::S(format!("USER#{}", username.to_uppercase())),
                )
                .item(
                    "SK",
                    AttributeValue::S(format!("TODO#{0}", &todo.id.to_uppercase())),
                )
                .item("text", AttributeValue::S(todo.text.to_string()))
                .item("id", AttributeValue::S(todo.id.to_string()))
                .item("completed", AttributeValue::Bool(todo.completed))
                .send()
                .await;
        }
        

        pub async fn complete(&self, username: String, id: &String) {
            let _res = &self
                .client
                .update_item()
                .table_name(&self.table_name)
                .key(
                    "PK",
                    AttributeValue::S(format!("USER#{}", username.to_uppercase())),
                )
                .key(
                    "SK",
                    AttributeValue::S(format!("TODO#{0}", &id.to_uppercase())),
                )
                .update_expression("SET completed = :completed")
                .expression_attribute_values(":completed", AttributeValue::Bool(true))
                .send()
                .await;
        }

        pub async fn delete_todo(&self, username: String, id: &String) {
            let _res = &self
                .client
                .delete_item()
                .table_name(&self.table_name)
                .key(
                    "PK",
                    AttributeValue::S(format!("USER#{}", username.to_uppercase())),
                )
                .key(
                    "SK",
                    AttributeValue::S(format!("TODO#{0}", &id.to_uppercase())),
                )
                .send()
                .await;
        }
    }

    #[derive(Debug, Serialize, Clone)]
    pub struct Todo {
        pub id: String,
        pub text: String,
        pub completed: bool,
    }

    pub struct TodoHomePageView {
        pub active: Vec<Todo>,
        pub completed: Vec<Todo>
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CreateTodo {
        pub text: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CompleteTodo {
        pub id: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct DeleteTodo {
        pub id: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct LoginCommand {
        pub username: String,
        pub password: String,
    }
}
