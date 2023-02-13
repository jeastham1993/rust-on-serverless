pub mod services {
    use aws_sdk_dynamodb::{Client, model::AttributeValue};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    pub struct TodoService {
        client: Client,
        table_name: String,
    }

    impl TodoService {
        pub fn new(client: Client, table_name: String) -> TodoService {
            TodoService {
                client: client,
                table_name: table_name,
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
                .into_iter()
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
        
            let res = &self
                .client
                .put_item()
                .table_name(&self.table_name)
                .item("PK", AttributeValue::S(format!("USER#{}", username.to_uppercase())))
                .item(
                    "SK",
                    AttributeValue::S(String::from(format!("TODO#{0}", &todo.id.to_uppercase()))),
                )
                .item("text", AttributeValue::S(todo.text.to_string()))
                .item("id", AttributeValue::S(todo.id.to_string()))
                .item("completed", AttributeValue::Bool(todo.completed))
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

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CreateTodo {
        pub text: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct LoginCommand {
        pub username: String,
        pub password: String
    }
}
