use crate::application::domain::ToDoRepo;
use crate::application::error_types::ValidationError;
use crate::application::events::{MessageType, ToDoCompleted, ToDoCreated, ToDoUpdated};
use crate::application::messaging::MessagePublisher;
use std::sync::Arc;

use super::{
    domain::{OwnerId, Title, ToDo},
    error_types::ServiceError,
    public_types::{CreateToDoCommand, ToDoItem, UpdateToDoCommand},
};

pub async fn create_to_do(
    owner: String,
    input: CreateToDoCommand,
    client: &Arc<dyn ToDoRepo + Send + Sync>,
    message_publisher: &Arc<dyn MessagePublisher + Send + Sync>,
) -> Result<ToDoItem, ValidationError> {
    let parsed_title = Title::new(input.title);
    let parsed_ownerid = OwnerId::new(owner);

    if parsed_title.is_err() || parsed_ownerid.is_err() {
        let mut errors = Vec::new();
        errors.push(parsed_title.err());
        errors.push(parsed_ownerid.err());

        return Err(combine_errors(errors));
    }

    let parsed_duedate = match input.due_date {
        None => None,
        Some(due_date) => {
            let parsed_due_date = chrono::DateTime::parse_from_rfc3339(due_date.as_str());

            match parsed_due_date {
                Ok(date) => Some(date),
                Err(_) => None,
            }
        }
    };

    let to_do = ToDo::new(
        parsed_title.unwrap(),
        parsed_ownerid.unwrap(),
        input.description,
        parsed_duedate,
    );

    match to_do {
        Ok(val) => {
            let db_res = client.create(&val).await;

            match db_res {
                Ok(_) => {
                    let _ = message_publisher
                        .publish(MessageType::ToDoCreated(ToDoCreated::new(
                            val.get_id(),
                            val.get_owner(),
                        )))
                        .await;

                    Ok(val.into_dto())
                }
                Err(_) => Err(ValidationError::new("Failure creating ToDo".to_string())),
            }
        }
        Err(e) => {
            let mut error_string = String::from("");

            for err in e {
                error_string = format!("{} {}", error_string, err.to_string());
            }

            Err(ValidationError::new(error_string))
        }
    }
}

pub async fn update_todo(
    owner: String,
    to_do_id: String,
    update_command: UpdateToDoCommand,
    client: &Arc<dyn ToDoRepo + Send + Sync>,
    message_publisher: &Arc<dyn MessagePublisher + Send + Sync>,
) -> Result<ToDoItem, ServiceError> {
    let query_res = client.get(&owner, &to_do_id).await;

    match query_res {
        Ok(todo) => {
            let updated_status = match update_command.set_as_complete {
                true => todo.set_completed(),
                false => todo,
            };

            let mut updated_todo = updated_status.update_title(update_command.title);

            match updated_todo {
                Ok(mut res) => {
                    res = res.update_description(update_command.description)
                        .update_due_date(update_command.due_date);

                    if (!res.has_changes()) {
                        return Ok(res.into_dto());
                    }

                    let database_result = client.create(&res).await;

                    let _ = match update_command.set_as_complete {
                        true => {
                            message_publisher
                                .publish(MessageType::ToDoCompleted(ToDoCompleted::new(
                                    res.get_id(),
                                    res.get_owner(),
                                )))
                                .await
                        }
                        false => {
                            message_publisher
                                .publish(MessageType::ToDoUpdated(ToDoUpdated::new(
                                    res.get_id(),
                                    res.get_owner(),
                                )))
                                .await
                        }
                    };

                    match database_result {
                        Ok(_) => Ok(res.into_dto()),
                        Err(e) => {
                            tracing::error!("{}", e.to_string());

                            Err(ServiceError::new(e.to_string()))
                        }
                    }
                }
                Err(e) => Err(ServiceError::new(e.to_string())),
            }
        }
        Err(e) => {
            tracing::error!("{}", e.to_string());

            Err(ServiceError::new(String::from("Record not found")))
        }
    }
}

fn combine_errors(err: Vec<Option<ValidationError>>) -> ValidationError {
    let mut error_builder = String::from("");

    for ele in err {
        error_builder = format!(
            "{} {}",
            error_builder,
            match ele {
                None => String::from(""),
                Some(val) => val.to_string(),
            }
        )
    }

    ValidationError::new(error_builder)
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
    use crate::application::{
        commands,
        domain::{OwnerId, Title, ToDo, ToDoId, ToDoRepo},
        error_types::RepositoryError,
        public_types::UpdateToDoCommand,
    };

    struct MockRepository {
        should_fail: bool,
        to_do_status_to_return: String,
    }

    #[async_trait]
    impl ToDoRepo for MockRepository {
        async fn list(&self, user_id: &str) -> Result<Vec<ToDo>, RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            }

            let mut todos: Vec<ToDo> = Vec::new();

            todos.push(
                ToDo::parse(
                    Title::new("title".to_string()).unwrap(),
                    OwnerId::new("owner".to_string()).unwrap(),
                    Some(self.to_do_status_to_return.to_string()),
                    Some(ToDoId::parse("id".to_string()).unwrap()),
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

        async fn create(&self, to_do: &ToDo) -> Result<(), RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            } else {
                Ok(())
            }
        }

        async fn get(&self, user_id: &str, todo_id: &str) -> Result<ToDo, RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            }

            Ok(ToDo::parse(
                Title::new("title".to_string()).unwrap(),
                OwnerId::new("owner".to_string()).unwrap(),
                Some(self.to_do_status_to_return.to_string()),
                Some(ToDoId::parse("id".to_string()).unwrap()),
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
    async fn update_todo_should_update_title() {
        let shared_state = Arc::new(AppState {
            todo_repo: Arc::new(MockRepository {
                should_fail: false,
                to_do_status_to_return: "INCOMPLETE".to_string(),
            }),
            message_publisher: Arc::new(InMemoryMessagePublisher::new()),
        });

        let to_dos = commands::update_todo(
            "jameseastham".to_string(),
            "12345".to_string(),
            UpdateToDoCommand {
                title: "newtitle".to_string(),
                set_as_complete: false,
                description: None,
                due_date: None,
            },
            &shared_state.todo_repo,
            &shared_state.message_publisher,
        )
        .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.unwrap().title, "newtitle");
    }

    #[tokio::test]
    async fn update_todo_should_update_description() {
        let shared_state = Arc::new(AppState {
            todo_repo: Arc::new(MockRepository {
                should_fail: false,
                to_do_status_to_return: "INCOMPLETE".to_string(),
            }),
            message_publisher: Arc::new(InMemoryMessagePublisher::new()),
        });

        let to_dos = commands::update_todo(
            "jameseastham".to_string(),
            "12345".to_string(),
            UpdateToDoCommand {
                title: "newtitle".to_string(),
                set_as_complete: false,
                description: Some("mydescription".to_string()),
                due_date: None,
            },
            &shared_state.todo_repo,
            &shared_state.message_publisher,
        )
            .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.unwrap().description, "mydescription");
    }

    #[tokio::test]
    async fn update_todo_should_update_due_date() {
        let shared_state = Arc::new(AppState {
            todo_repo: Arc::new(MockRepository {
                should_fail: false,
                to_do_status_to_return: "INCOMPLETE".to_string(),
            }),
            message_publisher: Arc::new(InMemoryMessagePublisher::new()),
        });

        let to_dos = commands::update_todo(
            "jameseastham".to_string(),
            "12345".to_string(),
            UpdateToDoCommand {
                title: "newtitle".to_string(),
                set_as_complete: false,
                description: Some("mydescription".to_string()),
                due_date: Some("2023-08-13T00:00:00+00:00".to_string()),
            },
            &shared_state.todo_repo,
            &shared_state.message_publisher,
        )
            .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.unwrap().due_date, "2023-08-13T00:00:00+00:00");
    }

    #[tokio::test]
    async fn update_completed_todo_title_should_not_change() {
        let shared_state = Arc::new(AppState {
            todo_repo: Arc::new(MockRepository {
                should_fail: false,
                to_do_status_to_return: "COMPLETE".to_string(),
            }),
            message_publisher: Arc::new(InMemoryMessagePublisher::new()),
        });

        let to_dos = commands::update_todo(
            "jameseastham".to_string(),
            "12345".to_string(),
            UpdateToDoCommand {
                title: "newtitle".to_string(),
                set_as_complete: true,
                description: None,
                due_date: None,
            },
            &shared_state.todo_repo,
            &shared_state.message_publisher,
        )
        .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.unwrap().title, "title");
    }

    #[tokio::test]
    async fn update_incomplete_todo_title_should_change() {
        let shared_state = Arc::new(AppState {
            todo_repo: Arc::new(MockRepository {
                should_fail: false,
                to_do_status_to_return: "INCOMPLETE".to_string(),
            }),
            message_publisher: Arc::new(InMemoryMessagePublisher::new()),
        });

        let to_dos = commands::update_todo(
            "jameseastham".to_string(),
            "12345".to_string(),
            UpdateToDoCommand {
                title: "newtitle".to_string(),
                set_as_complete: false,
                description: None,
                due_date: None,
            },
            &shared_state.todo_repo,
            &shared_state.message_publisher,
        )
        .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.unwrap().title, "newtitle");
    }

    #[tokio::test]
    async fn update_incomplete_todo_to_be_complete_should_set_complete() {
        let shared_state = Arc::new(AppState {
            todo_repo: Arc::new(MockRepository {
                should_fail: false,
                to_do_status_to_return: "INCOMPLETE".to_string(),
            }),
            message_publisher: Arc::new(InMemoryMessagePublisher::new()),
        });

        let to_dos = commands::update_todo(
            "jameseastham".to_string(),
            "12345".to_string(),
            UpdateToDoCommand {
                title: "newtitle".to_string(),
                set_as_complete: true,
                description: None,
                due_date: None,
            },
            &shared_state.todo_repo,
            &shared_state.message_publisher,
        )
        .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.as_ref().unwrap().title, "title");
        assert_eq!(to_dos.as_ref().unwrap().is_complete, true);
    }
}
