use std::any;

use crate::domain::{entities::Repository, error_types::ValidationError};

use super::{
    entities::{OwnerId, Title, ToDo, ToDoId},
    public_types::{CreateToDoCommand, ToDoItem, UpdateToDoCommand},
};

pub async fn create_to_do(
    input: CreateToDoCommand,
    client: &dyn Repository,
) -> Result<ToDoItem, ValidationError> {
    let parsed_title = Title::new(input.title);
    let parsed_ownerid = OwnerId::new(input.owner_id);

    if parsed_title.is_err() || parsed_ownerid.is_err() {
        let mut errors = Vec::new();
        errors.push(parsed_title.err());
        errors.push(parsed_ownerid.err());

        return Err(combine_errors(errors));
    }


    let to_do = ToDo::new(parsed_title.unwrap(), parsed_ownerid.unwrap());

    match to_do {
        Ok(val) => {
            let db_res = client.store_todo(&val).await;

            match db_res {
                Ok(_) => Ok(val.into_dto()),
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

pub async fn list_todos(owner: OwnerId, client: &dyn Repository) -> Result<Vec<ToDoItem>, ()> {
    let query_res = client.list_todos(&owner.to_string()).await;

    match query_res {
        Ok(todos) => {
            let mut to_do_items: Vec<ToDoItem> = Vec::new();

            for todo in todos {
                to_do_items.push(todo.into_dto());
            }

            Ok(to_do_items)
        }
        Err(_) => Err(()),
    }
}

pub async fn get_todos(
    owner: OwnerId,
    toDoId: ToDoId,
    client: &dyn Repository,
) -> Result<ToDoItem, ()> {
    let query_res = client
        .get_todo(&owner.to_string(), &toDoId.to_string())
        .await;

    match query_res {
        Ok(todo) => Ok(todo.into_dto()),
        Err(_) => Err(()),
    }
}

pub async fn update_todo(
    update_command: UpdateToDoCommand,
    client: &dyn Repository,
) -> Result<ToDoItem, ()> {
    let query_res = client
        .get_todo(&update_command.owner_id, &update_command.title)
        .await;

    match query_res {
        Ok(todo) => {
            let updated_status = todo.set_completed(update_command.set_as_complete);

            match updated_status {
                Ok(status_res) => {
                    let updated_todo = status_res.update_title(update_command.title);

                    match updated_todo {
                        Ok(res) => {
                            client.store_todo(&res).await;

                            Ok(res.into_dto())
                        }
                        Err(_) => Err(()),
                    }
                }
                Err(_) => Err(()),
            }
        }
        Err(_) => Err(()),
    }
}

fn combine_errors(err: Vec<Option<ValidationError>>) -> ValidationError {
    let mut error_builder = String::from("");

    for ele in err {
        error_builder = format!("{} {}", error_builder, match ele {
            Option::None => String::from(""),
            Some(val) => val.to_string()
        })
    }

    ValidationError::new(error_builder)
}

/// Unit tests
///
/// These tests are run using the `cargo test` command.
#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::domain::{
        entities::{OwnerId, Repository, Title, ToDo, ToDoId},
        error_types::RepositoryError,
        public_types::UpdateToDoCommand,
        todo_service,
    };

    struct MockRepository {
        should_fail: bool,
        to_do_status_to_return: String,
    }

    #[async_trait]
    impl Repository for MockRepository {
        async fn store_todo(&self, _: &ToDo) -> Result<(), RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            } else {
                Ok(())
            }
        }

        async fn get_todo(&self, _: &String, _: &String) -> Result<ToDo, RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            }

            Ok(ToDo::parse(
                Title::new("title".to_string()).unwrap(),
                OwnerId::new("owner".to_string()).unwrap(),
                Some(self.to_do_status_to_return.to_string()),
                Some(ToDoId::parse("id".to_string()).unwrap()),
            )
            .unwrap())
        }

        async fn list_todos(&self, _: &String) -> Result<Vec<ToDo>, RepositoryError> {
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
                )
                .unwrap(),
            );

            Ok(todos)
        }
    }

    #[tokio::test]
    async fn list_todos_should_return_todos() {
        let client = MockRepository {
            should_fail: false,
            to_do_status_to_return: "INCOMPLETE".to_string(),
        };

        let to_dos = todo_service::list_todos(OwnerId::new("owner".to_string()).unwrap(), &client).await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn get_tods_should_return_todo() {
        let client = MockRepository {
            should_fail: false,
            to_do_status_to_return: "INCOMPLETE".to_string(),
        };

        let to_dos = todo_service::get_todos(
            OwnerId::new("owner".to_string()).unwrap(),
            ToDoId::parse("the id".to_string()).unwrap(),
            &client,
        )
        .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.unwrap().title, "title");
    }

    #[tokio::test]
    async fn list_todos_on_error_should_return_eror() {
        let client = MockRepository {
            should_fail: true,
            to_do_status_to_return: "INCOMPLETE".to_string(),
        };

        let to_dos = todo_service::list_todos(OwnerId::new("owner".to_string()).unwrap(), &client).await;

        assert_eq!(to_dos.is_err(), true);
    }

    #[tokio::test]
    async fn update_todo_should_update_title() {
        let client = MockRepository {
            should_fail: false,
            to_do_status_to_return: "INCOMPLETE".to_string(),
        };

        let to_dos = todo_service::update_todo(
            UpdateToDoCommand {
                owner_id: "jameseastham".to_string(),
                title: "newtitle".to_string(),
                toDoId: "12345".to_string(),
                set_as_complete: false,
            },
            &client,
        )
        .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.unwrap().title, "newtitle");
    }

    #[tokio::test]
    async fn update_completed_todo_title_should_not_change() {
        let client = MockRepository {
            should_fail: false,
            to_do_status_to_return: "COMPLETE".to_string(),
        };

        let to_dos = todo_service::update_todo(
            UpdateToDoCommand {
                owner_id: "jameseastham".to_string(),
                title: "newtitle".to_string(),
                toDoId: "12345".to_string(),
                set_as_complete: true,
            },
            &client,
        )
        .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.unwrap().title, "title");
    }

    #[tokio::test]
    async fn update_incomplete_todo_title_should_change() {
        let client = MockRepository {
            should_fail: false,
            to_do_status_to_return: "INCOMPLETE".to_string(),
        };

        let to_dos = todo_service::update_todo(
            UpdateToDoCommand {
                owner_id: "jameseastham".to_string(),
                title: "newtitle".to_string(),
                toDoId: "12345".to_string(),
                set_as_complete: false,
            },
            &client,
        )
        .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.unwrap().title, "newtitle");
    }

    #[tokio::test]
    async fn update_incomplete_todo_to_be_complete_should_set_complete() {
        let client = MockRepository {
            should_fail: false,
            to_do_status_to_return: "INCOMPLETE".to_string(),
        };

        let to_dos = todo_service::update_todo(
            UpdateToDoCommand {
                owner_id: "jameseastham".to_string(),
                title: "newtitle".to_string(),
                toDoId: "12345".to_string(),
                set_as_complete: true,
            },
            &client,
        )
        .await;

        assert_eq!(to_dos.is_err(), false);
        assert_eq!(to_dos.as_ref().unwrap().title, "title");
        assert_eq!(to_dos.as_ref().unwrap().is_complete, true);
    }
}
