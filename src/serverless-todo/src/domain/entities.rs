use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use super::{
    error_types::{RepositoryError, ValidationError},
    public_types::ToDoItem,
};

const INCOMPLETE_STATUS: &str = "INCOMPLETE";
const COMPLETE_STATUS: &str = "COMPLETE";

/// Represents a ToDo list item, a ToDo can be incomplete or complete.
#[non_exhaustive]
pub enum ToDo {
    /// Represents an incomplete ToDo item
    Incomplete(IncompleteToDo),
    /// Represents a complete ToDo item
    Complete(CompleteToDo),
}

impl ToDo {
    /// Create a new ToDo item from a title and owner. 
    /// Returns a new IncompleteToDo
    pub fn new(title: Title, owner_id: OwnerId) -> Result<ToDo, Vec<ValidationError>> {
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

        let id = ToDoId::new();

        Ok(ToDo::Incomplete(IncompleteToDo {
            to_do_id: id,
            title: title,
            owner: owner_id,
        }))
    }

    /// Parse a ToDo from a set of existing values
    pub fn parse(
        title: Title,
        owner_id: OwnerId,
        status: Option<String>,
        existing_id: Option<ToDoId>,
        completed_on: Option<DateTime<FixedOffset>>,
    ) -> Result<ToDo, Vec<ValidationError>> {
        let mut errors: Vec<ValidationError> = Vec::new();

        let title_res = ToDo::check_title(&title);
        let owner_res = ToDo::check_owner_id(&owner_id);

        if title_res.is_err() || owner_res.is_err() {
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
            Option::None => ToDoId::new(),
            Option::Some(val) => ToDoId::parse(val.to_string()).unwrap(),
        };

        match status {
            Option::Some(status_val) => {
                match status_val.as_str() {
                    INCOMPLETE_STATUS => Ok(ToDo::Incomplete(IncompleteToDo {
                        to_do_id: existing_id.unwrap(),
                        title: title,
                        owner: owner_id,
                    })),
                    COMPLETE_STATUS => {
                        let parsed_completed_on = match completed_on {
                            Option::None => {
                                errors.push(ValidationError::new("Is status is completed a valid completed on date must be passed".to_string()));
                                return Err(errors);
                            }
                            Some(val) => val,
                        };

                        Ok(ToDo::Complete(CompleteToDo {
                            to_do_id: existing_id.unwrap(),
                            title: title,
                            owner: owner_id,
                            completed_on: parsed_completed_on,
                        }))
                    }
                    _ => Ok(ToDo::Incomplete(IncompleteToDo {
                        to_do_id: id,
                        title: title,
                        owner: owner_id,
                    })),
                }
            }
            _ => Ok(ToDo::Incomplete(IncompleteToDo {
                to_do_id: id,
                title: title,
                owner: owner_id,
            })),
        }
    }

    /// GET the title of the ToDo
    pub fn get_title(&self) -> String {
        match &self {
            ToDo::Incomplete(incomplete) => incomplete.title.to_string(),
            ToDo::Complete(complete) => complete.title.to_string(),
        }
    }

    /// GET the date the ToDo was completed. Returns an empty string if incomplete.
    pub fn get_completed_on(&self) -> String {
        match &self {
            ToDo::Incomplete(_) => "".to_string(),
            ToDo::Complete(complete) => complete.completed_on.to_rfc3339(),
        }
    }

    /// GET the owner of the ToDo
    pub fn get_owner(&self) -> String {
        match &self {
            ToDo::Incomplete(incomplete) => incomplete.owner.to_string(),
            ToDo::Complete(complete) => complete.owner.to_string(),
        }
    }

    /// GET the ID of the ToDo
    pub fn get_id(&self) -> String {
        match &self {
            ToDo::Incomplete(incomplete) => incomplete.to_do_id.to_string(),
            ToDo::Complete(complete) => complete.to_do_id.to_string(),
        }
    }

    /// GET the status of the ToDo
    pub fn get_status(&self) -> String {
        match &self {
            ToDo::Incomplete(_) => String::from(INCOMPLETE_STATUS),
            ToDo::Complete(_) => String::from(COMPLETE_STATUS),
        }
    }

    /// Update the title of the existing ToDo.
    /// If the ToDo is already completed then the title cannot be updated.
    /// Returns a new ToDo
    pub fn update_title(self, new_title: String) -> Result<ToDo, ValidationError> {
        let new_title_value = Title::new(new_title);

        if new_title_value.is_err() {
            return Err(new_title_value.err().unwrap());
        }

        let response = match &self {
            ToDo::Incomplete(incomplete) => ToDo::Incomplete(IncompleteToDo {
                to_do_id: incomplete.to_do_id.clone(),
                title: new_title_value.unwrap(),
                owner: OwnerId::new(incomplete.owner.to_string()).unwrap(),
            }),
            ToDo::Complete(complete) => ToDo::Complete(CompleteToDo {
                to_do_id: complete.to_do_id.clone(),
                title: Title::new(complete.title.to_string()).unwrap(),
                owner: OwnerId::new(complete.owner.to_string()).unwrap(),
                completed_on: complete.completed_on,
            }),
        };

        Ok(response)
    }

    /// Set the ToDo as completed
    pub fn set_completed(self) -> ToDo {
        match &self {
            ToDo::Incomplete(incomplete) => ToDo::Complete(CompleteToDo {
                to_do_id: incomplete.to_do_id.clone(),
                title: Title::new(incomplete.title.to_string()).unwrap(),
                owner: OwnerId::new(incomplete.owner.to_string()).unwrap(),
                completed_on: DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap(),
            }),
            ToDo::Complete(complete) => ToDo::Complete(CompleteToDo {
                to_do_id: complete.to_do_id.clone(),
                title: Title::new(complete.title.to_string()).unwrap(),
                owner: OwnerId::new(complete.owner.to_string()).unwrap(),
                completed_on: complete.completed_on,
            }),
        }
    }

    /// Convert the ToDo into a ToDoItem Data Transfer Object
    pub fn into_dto(self) -> ToDoItem {
        match &self {
            ToDo::Incomplete(incomplete) => ToDoItem {
                id: incomplete.to_do_id.to_string(),
                is_complete: false,
                title: incomplete.title.to_string(),
                completed_on: String::from("")
            },
            ToDo::Complete(complete) => ToDoItem {
                id: complete.to_do_id.to_string(),
                is_complete: true,
                title: complete.title.to_string(),
                completed_on: complete.completed_on.to_rfc3339()
            },
        }
    }

    fn check_title(input: &Title) -> Result<(), ValidationError> {
        tracing::info!("Checking title: '{}'", input.to_string());

        if input.to_string().len() <= 0 || input.to_string().len() > 50 {
            tracing::info!("Title is invalid");

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

/// Represents the structure of an incomplete ToDo
#[non_exhaustive]
pub struct IncompleteToDo {
    to_do_id: ToDoId,
    title: Title,
    owner: OwnerId,
}

/// Represents the structure of a complete ToDo item
#[non_exhaustive]
pub struct CompleteToDo {
    to_do_id: ToDoId,
    title: Title,
    owner: OwnerId,
    completed_on: DateTime<FixedOffset>,
}

#[derive(Clone)]
pub struct ToDoId {
    value: String,
}

impl ToDoId {
    pub fn new() -> ToDoId {
        ToDoId::parse(Uuid::new_v4().to_string()).unwrap()
    }

    pub fn parse(existing_id: String) -> Result<ToDoId, ValidationError> {
        if existing_id.to_string().len() <= 0 || existing_id.to_string().len() > 50 {
            tracing::info!("Title is invalid");

            return Err(ValidationError::new(
                "Must be between 1 and 50 chars".to_string(),
            ));
        }

        Ok(ToDoId {
            value: existing_id.to_string(),
        })
    }

    pub fn to_string(&self) -> String {
        self.value.clone()
    }
}

#[derive(Clone)]
pub struct Title {
    value: String,
}

impl Title {
    pub fn new(title: String) -> Result<Title, ValidationError> {
        if title.to_string().len() <= 0 || title.to_string().len() > 50 {
            tracing::info!("Title is invalid");

            return Err(ValidationError::new(
                "Must be between 1 and 50 chars".to_string(),
            ));
        }

        Ok(Title {
            value: title.to_string(),
        })
    }

    pub fn to_string(&self) -> String {
        self.value.clone()
    }
}

#[derive(Clone)]
pub struct OwnerId {
    value: String,
}

impl OwnerId {
    pub fn new(owner_id: String) -> Result<OwnerId, ValidationError> {
        if owner_id.to_string().len() <= 0 {
            tracing::info!("Title is invalid");

            return Err(ValidationError::new(
                "Must be between 1 and 50 chars".to_string(),
            ));
        }

        Ok(OwnerId {
            value: owner_id.to_string(),
        })
    }

    pub fn to_string(&self) -> String {
        self.value.clone()
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
    use chrono::{DateTime, Utc};

    use crate::domain::entities::{OwnerId, Title, ToDo};

    use super::ToDoId;

    #[test]
    fn valid_data_should_return_validated_to_do() {
        let to_do = ToDo::new(
            Title::new(String::from("my title")).unwrap(),
            OwnerId::new(String::from("jameseastham")).unwrap(),
        );

        assert_eq!(to_do.is_err(), false);
        assert_eq!(to_do.as_ref().unwrap().get_title(), "my title");
        assert_eq!(to_do.as_ref().unwrap().get_owner(), "jameseastham");
    }

    #[test]
    fn update_title_for_incomplete_todo_should_change() {
        let todo = ToDo::Incomplete(super::IncompleteToDo {
            to_do_id: ToDoId::parse(String::from("hello")).unwrap(),
            title: Title::new(String::from("hello")).unwrap(),
            owner: OwnerId::new(String::from("hello")).unwrap(),
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
            to_do_id: ToDoId::parse(String::from("hello")).unwrap(),
            title: Title::new(String::from("hello")).unwrap(),
            owner: OwnerId::new(String::from("hello")).unwrap(),
            completed_on: DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap(),
        });

        let updated_todo = todo.update_title(String::from("my new title"));

        if let ToDo::Complete(completed) = updated_todo.unwrap() {
            assert_eq!(completed.title.to_string(), String::from("hello"))
        } else {
            panic!("ToDo update method did not return the expected type")
        }
    }

    #[test]
    fn update_status_for_incomplete_todo_should_change() {
        let todo = ToDo::Incomplete(super::IncompleteToDo {
            to_do_id: ToDoId::parse(String::from("hello")).unwrap(),
            title: Title::new(String::from("hello")).unwrap(),
            owner: OwnerId::new(String::from("hello")).unwrap(),
        });

        let updated_todo = todo.set_completed();

        if let ToDo::Complete(completed) = updated_todo {
            assert_eq!(completed.title.to_string(), String::from("hello"));
        } else {
            panic!("ToDo update method did not return the expected type")
        }
    }

    #[test]
    fn update_status_for_completed_todo_should_not_change() {
        let date = DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap();

        let todo = ToDo::Complete(super::CompleteToDo {
            to_do_id: ToDoId::parse(String::from("hello")).unwrap(),
            title: Title::new(String::from("hello")).unwrap(),
            owner: OwnerId::new(String::from("hello")).unwrap(),
            completed_on: date,
        });

        let updated_todo = todo.set_completed();

        if let ToDo::Complete(completed) = updated_todo {
            assert_eq!(completed.title.to_string(), String::from("hello"));
            assert_eq!(completed.completed_on, date);
        } else {
            panic!("ToDo update method did not return the expected type")
        }
    }

    #[test]
    fn new_id_should_return_valid_to_do_id() {
        let to_do_id = ToDoId::new();

        assert_eq!(to_do_id.to_string().len(), 36)
    }

    #[test]
    fn parse_empty_id_should_return_validate_error() {
        let to_do_id = ToDoId::parse(String::from(""));

        assert_eq!(to_do_id.is_err(), true);
    }

    #[test]
    fn empty_title_should_return_validate_error() {
        let to_do = Title::new(String::from(""));

        assert_eq!(to_do.is_err(), true);
    }

    #[test]
    fn empty_owner_should_return_validate_error() {
        let owner = OwnerId::new(String::from(""));

        assert_eq!(owner.is_err(), true);
    }
}
