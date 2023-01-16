use std::{fmt};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    error_types::{RepositoryError, ValidationError},
    public_types::{ToDoItem},
};

const UNVALIDATED_STATUS: &str = "UNVALIDATED";
const VALIDATED_STATUS: &str = "VALIDATED";
const INCOMPLETE_STATUS: &str = "INCOMPLETE";
const COMPLETE_STATUS: &str = "COMPLETE";

#[non_exhaustive]
pub enum ToDo {
    Unvalidated(UnvalidatedToDo),
    Validated(ValidatedToDo),
    Incomplete(IncompleteToDo),
    Complete(CompleteToDo),
}

impl ToDo {
    pub fn new(title: Title, owner_id: OwnerId, status: Option<String>, existing_id: Option<ToDoId>) -> Result<ToDo, Vec<ValidationError>> {
        let title_res = ToDo::check_title(&title);
        let owner_res = ToDo::check_owner_id(&owner_id);

        if title_res.is_err() || owner_res.is_err() {
            let mut errors: Vec<ValidationError> = Vec::new();
            let title_err = title_res.err();
            let owner_err = owner_res.err();

            if title_err.is_some() {
                errors.push(title_err.unwrap());
            }

            if owner_err.is_some() {
                errors.push(owner_err.unwrap());
            }

            return Err(errors);
        }
        
        let id = match &existing_id {
            Option::None => ToDoId::ToDoId(Uuid::new_v4().to_string()),
            Option::Some(val) => ToDoId::ToDoId(val.to_string())
        };

        match status {
            Option::Some(status_val) => {
                match status_val.as_str() {
                    UNVALIDATED_STATUS => Ok(ToDo::Unvalidated(UnvalidatedToDo { title: title, owner: owner_id })),
                    VALIDATED_STATUS => Ok(ToDo::Validated(ValidatedToDo { to_do_id: existing_id.unwrap(), title: title, owner: owner_id })),
                    INCOMPLETE_STATUS => Ok(ToDo::Incomplete(IncompleteToDo { to_do_id: existing_id.unwrap(), title: title, owner: owner_id })),
                    COMPLETE_STATUS => Ok(ToDo::Complete(CompleteToDo { to_do_id: existing_id.unwrap(), title: title, owner: owner_id })),
                    _ => Ok(ToDo::Unvalidated(UnvalidatedToDo { title: title, owner: owner_id })),
                }
            }
            _ => Ok(ToDo::Validated(ValidatedToDo { to_do_id: id, title: title, owner: owner_id })),
        }
    }

    pub fn get_title(&self) -> String {
        match &self {
            ToDo::Unvalidated(unvalidated) => unvalidated.title.to_string(),
            ToDo::Validated(validated) => validated.title.to_string(),
            ToDo::Incomplete(incomplete) => incomplete.title.to_string(),
            ToDo::Complete(complete) => complete.title.to_string(),
        }
    }

    pub fn get_owner(&self) -> String {
        match &self {
            ToDo::Unvalidated(unvalidated) => unvalidated.owner.to_string(),
            ToDo::Validated(validated) => validated.owner.to_string(),
            ToDo::Incomplete(incomplete) => incomplete.owner.to_string(),
            ToDo::Complete(complete) => complete.owner.to_string(),
        }
    }

    pub fn get_id(&self) -> String {
        match &self {
            ToDo::Unvalidated(_) => String::from(""),
            ToDo::Validated(validated) => validated.to_do_id.to_string(),
            ToDo::Incomplete(incomplete) => incomplete.to_do_id.to_string(),
            ToDo::Complete(complete) => complete.to_do_id.to_string(),
        }
    }

    pub fn get_status(&self) -> String {
        match &self {
            ToDo::Unvalidated(_) => String::from(UNVALIDATED_STATUS),
            ToDo::Validated(_) => String::from(VALIDATED_STATUS),
            ToDo::Incomplete(_) => String::from(INCOMPLETE_STATUS),
            ToDo::Complete(_) => String::from(COMPLETE_STATUS),
        }
    }

    // Forcing immutability. When the title of a ToDo needs to be updated a new ToDo is returned.
    pub fn update_title(self, new_title: String) -> Result<ToDo, ()> {
        let response = match &self {
            ToDo::Unvalidated(unvalidated) => ToDo::Unvalidated(UnvalidatedToDo {
                title: Title::Title(new_title),
                owner: OwnerId::OwnerId(unvalidated.owner.to_string()),
            }),
            ToDo::Validated(validated) => ToDo::Validated(ValidatedToDo {
                to_do_id: ToDoId::ToDoId(validated.to_do_id.to_string()),
                title: Title::Title(new_title),
                owner: OwnerId::OwnerId(validated.owner.to_string()),
            }),
            ToDo::Incomplete(incomplete) => ToDo::Incomplete(IncompleteToDo {
                to_do_id: ToDoId::ToDoId(incomplete.to_do_id.to_string()),
                title: Title::Title(new_title),
                owner: OwnerId::OwnerId(incomplete.owner.to_string()),
            }),
            ToDo::Complete(complete) => ToDo::Complete(CompleteToDo {
                to_do_id: ToDoId::ToDoId(complete.to_do_id.to_string()),
                title: Title::Title(complete.title.to_string()),
                owner: OwnerId::OwnerId(complete.owner.to_string()),
            }),
        };

        Ok(response)
    }

    pub fn set_completed(self, is_complete: bool) -> Result<ToDo, ()> {
        let response = match is_complete {
            true => match &self {
                ToDo::Unvalidated(unvalidated) => ToDo::Unvalidated(UnvalidatedToDo {
                    title: Title::Title(unvalidated.title.to_string()),
                    owner: OwnerId::OwnerId(unvalidated.owner.to_string()),
                }),
                ToDo::Validated(validated) => ToDo::Complete(CompleteToDo {
                    to_do_id: ToDoId::ToDoId(validated.to_do_id.to_string()),
                    title: Title::Title(validated.title.to_string()),
                    owner: OwnerId::OwnerId(validated.owner.to_string()),
                }),
                ToDo::Incomplete(incomplete) => ToDo::Complete(CompleteToDo {
                    to_do_id: ToDoId::ToDoId(incomplete.to_do_id.to_string()),
                    title: Title::Title(incomplete.title.to_string()),
                    owner: OwnerId::OwnerId(incomplete.owner.to_string()),
                }),
                ToDo::Complete(complete) => ToDo::Complete(CompleteToDo {
                    to_do_id: ToDoId::ToDoId(complete.to_do_id.to_string()),
                    title: Title::Title(complete.title.to_string()),
                    owner: OwnerId::OwnerId(complete.owner.to_string()),
                }),
            },
            false => match &self {
                ToDo::Unvalidated(unvalidated) => ToDo::Unvalidated(UnvalidatedToDo {
                    title: Title::Title(unvalidated.title.to_string()),
                    owner: OwnerId::OwnerId(unvalidated.owner.to_string()),
                }),
                ToDo::Validated(validated) => ToDo::Incomplete(IncompleteToDo {
                    to_do_id: ToDoId::ToDoId(validated.to_do_id.to_string()),
                    title: Title::Title(validated.title.to_string()),
                    owner: OwnerId::OwnerId(validated.owner.to_string()),
                }),
                ToDo::Incomplete(incomplete) => ToDo::Incomplete(IncompleteToDo {
                    to_do_id: ToDoId::ToDoId(incomplete.to_do_id.to_string()),
                    title: Title::Title(incomplete.title.to_string()),
                    owner: OwnerId::OwnerId(incomplete.owner.to_string()),
                }),
                ToDo::Complete(complete) => ToDo::Incomplete(IncompleteToDo {
                    to_do_id: ToDoId::ToDoId(complete.to_do_id.to_string()),
                    title: Title::Title(complete.title.to_string()),
                    owner: OwnerId::OwnerId(complete.owner.to_string()),
                }),
            }
        };

        Ok(response)
    }

    pub fn into_dto(self) -> ToDoItem {
        match &self {
            ToDo::Unvalidated(unvalidated) => ToDoItem {
                id: String::from(""),
                is_complete: false,
                title: unvalidated.title.to_string(),
            },
            ToDo::Validated(validated) => ToDoItem {
                id: validated.to_do_id.to_string(),
                is_complete: false,
                title: validated.title.to_string(),
            },
            ToDo::Incomplete(incomplete) => ToDoItem {
                id: incomplete.to_do_id.to_string(),
                is_complete: false,
                title: incomplete.title.to_string(),
            },
            ToDo::Complete(complete) => ToDoItem {
                id: complete.to_do_id.to_string(),
                is_complete: true,
                title: complete.title.to_string(),
            },
        }
    }

    fn check_title(input: &Title) -> Result<(), ValidationError> {
        println!("Checking title: '{}'", input.to_string());

        if input.to_string().len() <= 0 || input.to_string().len() > 50 {
            println!("Title is invalid");

            return Err(ValidationError::new(
                "Must be between 1 and 50 chars".to_string(),
            ));
        }

        Ok(())
    }

    fn check_owner_id(input: &OwnerId) -> Result<(), ValidationError> {
        if input.to_string().len() <= 0 {
            return Err(ValidationError::new(
                "Owner Id must have a length".to_string(),
            ));
        }

        Ok(())
    }
}

#[non_exhaustive]
pub struct UnvalidatedToDo {
    title: Title,
    owner: OwnerId,
}

#[non_exhaustive]
pub struct ValidatedToDo {
    to_do_id: ToDoId,
    title: Title,
    owner: OwnerId,
}

#[non_exhaustive]
pub struct IncompleteToDo {
    to_do_id: ToDoId,
    title: Title,
    owner: OwnerId,
}

#[non_exhaustive]
pub struct CompleteToDo {
    to_do_id: ToDoId,
    title: Title,
    owner: OwnerId,
}

pub enum ToDoId {
    ToDoId(String),
}

impl ToDoId {
    pub fn to_string(&self) -> String {
        let ToDoId::ToDoId(value) = self;

        value.clone()
    }
}

pub enum Title {
    Title(String),
}

impl Title {
    pub fn to_string(&self) -> String {
        let Title::Title(value) = self;

        value.clone()
    }
}

pub enum OwnerId {
    OwnerId(String),
}

impl OwnerId {
    pub fn to_string(&self) -> String {
        let OwnerId::OwnerId(value) = self;

        value.clone()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum IsComplete {
    INCOMPLETE,
    COMPLETE,
}

impl fmt::Display for IsComplete {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[async_trait]
pub trait Repository {
    async fn store_todo(&self, body: &ToDo) -> Result<(), RepositoryError>;

    async fn get_todo(&self, owner: &String, id: &String) -> Result<ToDo, RepositoryError>;

    async fn list_todos(&self, owner: &String) -> Result<Vec<ToDo>, RepositoryError>;
}

/// Unit tests
///
/// These tests are run using the `cargo test` command.
#[cfg(test)]
mod tests {
    use crate::domain::entities::{ToDo, OwnerId, Title};

    use super::ToDoId;

    #[test]
    fn valid_data_should_return_validated_to_do() {
        let to_do = ToDo::new(Title::Title(String::from("my title")),
            OwnerId::OwnerId(String::from("jameseastham")),
            Option::None,
            Option::None
        );

        assert_eq!(to_do.is_err(), false);
        assert_eq!(to_do.as_ref().unwrap().get_title(), "my title");
        assert_eq!(to_do.as_ref().unwrap().get_owner(), "jameseastham");
    }

    #[test]
    fn update_title_for_incomplete_todo_should_change() {
        let todo = ToDo::Incomplete(super::IncompleteToDo {
            to_do_id: ToDoId::ToDoId(String::from("hello")),
            title: Title::Title(String::from("hello")),
            owner: OwnerId::OwnerId(String::from("hello")),
        });

        let updated_todo = todo.update_title(String::from("my new title"));

        if let ToDo::Incomplete(todo) = updated_todo.unwrap() {
            assert_eq!(todo.title.to_string(), String::from("my new title"))
        } else {
            panic!("ToDo update method did not return the expected type")
        }
    }

    #[test]
    fn update_title_for_completed_todo_should_not_change() {
        let todo = ToDo::Complete(super::CompleteToDo {
            to_do_id: ToDoId::ToDoId(String::from("hello")),
            title: Title::Title(String::from("hello")),
            owner: OwnerId::OwnerId(String::from("hello")),
        });

        let updated_todo = todo.update_title(String::from("my new title"));

        if let ToDo::Complete(completed) = updated_todo.unwrap() {
            assert_eq!(completed.title.to_string(), String::from("hello"))
        } else {
            panic!("ToDo update method did not return the expected type")
        }
    }

    #[test]
    fn empty_title_should_return_validate_error() {
        let to_do = ToDo::new(Title::Title(String::from("")),
            OwnerId::OwnerId(String::from("jameseastham")),
            Option::None,
            Option::None
        );

        assert_eq!(to_do.is_err(), true);
    }

    #[test]
    fn empty_owner_should_return_validate_error() {
        let to_do = ToDo::new(Title::Title(String::from("my title")),
            OwnerId::OwnerId(String::from("")),
            Option::None,
            Option::None
        );

        assert_eq!(to_do.is_err(), true);
    }
}
