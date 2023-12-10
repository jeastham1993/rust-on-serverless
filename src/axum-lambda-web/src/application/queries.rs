use crate::application::domain::ToDoRepo;
use crate::application::public_types::ToDoItem;
use std::sync::Arc;

pub async fn list_todos(
    owner: &String,
    client: &Arc<dyn ToDoRepo + Send + Sync>,
) -> Result<Vec<ToDoItem>, ()> {
    let query_res = client.list(owner).await;

    match query_res {
        Ok(todos) => {
            let mut to_do_items: Vec<ToDoItem> = Vec::new();

            for todo in todos {
                to_do_items.push(todo.as_dto());
            }

            Ok(to_do_items)
        }
        Err(_) => Err(()),
    }
}

pub async fn get_todos(
    owner: &String,
    to_do_id: &str,
    client: &Arc<dyn ToDoRepo + Send + Sync>,
) -> Result<ToDoItem, ()> {
    let query_res = client.get(owner, to_do_id).await;

    match query_res {
        Ok(todo) => Ok(todo.as_dto()),
        Err(_) => Err(()),
    }
}

/// Unit tests
///
/// These tests are run using the `cargo test` command.
#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use std::sync::Arc;

    use crate::application::domain::AppState;
    use crate::application::messaging::InMemoryMessagePublisher;
    use crate::application::queries::{get_todos, list_todos};
    use crate::application::{
        domain::{OwnerId, Title, ToDo, ToDoId, ToDoRepo},
        error_types::RepositoryError,
    };

    struct MockRepository {
        should_fail: bool,
        to_do_status_to_return: String,
    }

    #[async_trait]
    impl ToDoRepo for MockRepository {
        async fn list(&self, _user_id: &str) -> Result<Vec<ToDo>, RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            }

            let mut todos: Vec<ToDo> = Vec::new();

            todos.push(
                ToDo::parse(
                    Title::new("title").unwrap(),
                    OwnerId::new("owner").unwrap(),
                    Some(self.to_do_status_to_return.to_string()),
                    Some(ToDoId::parse("id").unwrap()),
                    Some(String::from("Description")),
                    Some(DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap()),
                    match self.to_do_status_to_return.as_str() {
                        "COMPLETE" => {
                            Some(DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap())
                        }
                        _ => None,
                    },
                )
                .unwrap(),
            );

            Ok(todos)
        }

        async fn create(&self, _to_do: &ToDo) -> Result<(), RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            } else {
                Ok(())
            }
        }

        async fn get(&self, _user_id: &str, _todo_id: &str) -> Result<ToDo, RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            }

            Ok(ToDo::parse(
                Title::new("title").unwrap(),
                OwnerId::new("owner").unwrap(),
                Some(self.to_do_status_to_return.to_string()),
                Some(ToDoId::parse("id").unwrap()),
                Some(String::from("Description")),
                Some(DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap()),
                match self.to_do_status_to_return.as_str() {
                    "COMPLETE" => {
                        Some(DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap())
                    }
                    _ => None,
                },
            )
            .unwrap())
        }
    }

    #[tokio::test]
    async fn list_todos_should_return_todos() {
        let shared_state = Arc::new(AppState {
            todo_repo: Arc::new(MockRepository {
                should_fail: false,
                to_do_status_to_return: "INCOMPLETE".to_string(),
            }),
            message_publisher: Arc::new(InMemoryMessagePublisher::new()),
        });

        let to_dos = list_todos(&String::from("owner"), &shared_state.todo_repo).await;

        assert!(!to_dos.is_err());
        assert_eq!(to_dos.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn get_todos_should_return_todo() {
        let shared_state = Arc::new(AppState {
            todo_repo: Arc::new(MockRepository {
                should_fail: false,
                to_do_status_to_return: "INCOMPLETE".to_string(),
            }),
            message_publisher: Arc::new(InMemoryMessagePublisher::new()),
        });

        let to_dos = get_todos(&String::from("owner"), "the id", &shared_state.todo_repo).await;

        assert!(!to_dos.is_err());
        assert_eq!(to_dos.unwrap().title, "title");
    }

    #[tokio::test]
    async fn list_todos_on_error_should_return_error() {
        let shared_state = Arc::new(AppState {
            todo_repo: Arc::new(MockRepository {
                should_fail: true,
                to_do_status_to_return: "INCOMPLETE".to_string(),
            }),
            message_publisher: Arc::new(InMemoryMessagePublisher::new()),
        });

        let to_dos = list_todos(&String::from("owner"), &shared_state.todo_repo).await;

        assert!(to_dos.is_err());
    }
}
