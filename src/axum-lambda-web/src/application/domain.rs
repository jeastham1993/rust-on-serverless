use crate::application::helpers::check_not_empty_and_length_less_than;
use crate::application::messaging::MessagePublisher;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

use super::error_types::{RepositoryError, ValidationError};

pub struct AppState {
    pub todo_repo: Arc<dyn ToDoRepo + Send + Sync>,
    pub message_publisher: Arc<dyn MessagePublisher + Send + Sync>,
}

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
    pub(crate) fn new(
        title: Title,
        owner_id: OwnerId,
        description: Option<String>,
        due_date: Option<DateTime<FixedOffset>>,
    ) -> Result<ToDo, Vec<ValidationError>> {
        let title_res = ToDo::check_title(&title);
        let owner_res = ToDo::check_owner_id(&owner_id);

        if title_res.is_err() || owner_res.is_err() {
            let mut errors: Vec<ValidationError> = Vec::new();
            let title_err = title_res.err();
            let owner_err = owner_res.err();

            if let Some(err) = title_err {
                errors.push(err);
            }

            if let Some(err) = owner_err {
                errors.push(err);
            }

            return Err(errors);
        }

        let id = ToDoId::new();

        Ok(ToDo::Incomplete(IncompleteToDo {
            to_do_id: id,
            title,
            owner: owner_id,
            description,
            due_date,
            has_changes: false,
        }))
    }

    pub(crate) fn has_changes(&self) -> bool {
        match &self {
            ToDo::Incomplete(incomplete) => incomplete.has_changes,
            ToDo::Complete(complete) => complete.has_changes,
        }
    }

    /// Parse a ToDo from a set of existing values
    pub(crate) fn parse(
        title: Title,
        owner_id: OwnerId,
        status: Option<String>,
        existing_id: Option<ToDoId>,
        description: Option<String>,
        due_date: Option<DateTime<FixedOffset>>,
        completed_on: Option<DateTime<FixedOffset>>,
    ) -> Result<ToDo, Vec<ValidationError>> {
        let mut errors: Vec<ValidationError> = Vec::new();

        let title_res = ToDo::check_title(&title);
        let owner_res = ToDo::check_owner_id(&owner_id);

        if title_res.is_err() || owner_res.is_err() {
            let title_err = title_res.err();
            let owner_err = owner_res.err();

            if let Some(err) = title_err {
                errors.push(err);
            }

            if let Some(err) = owner_err {
                errors.push(err);
            }

            return Err(errors);
        }

        let id = match &existing_id {
            None => ToDoId::new(),
            Some(val) => ToDoId::parse(val.to_string()).unwrap(),
        };

        match status {
            Some(status_val) => {
                match status_val.as_str() {
                    INCOMPLETE_STATUS => Ok(ToDo::Incomplete(IncompleteToDo {
                        to_do_id: existing_id.unwrap(),
                        title,
                        owner: owner_id,
                        description,
                        due_date,
                        has_changes: false,
                    })),
                    COMPLETE_STATUS => {
                        let parsed_completed_on = match completed_on {
                            None => {
                                errors.push(ValidationError::new("Is status is completed a valid completed on date must be passed".to_string()));
                                return Err(errors);
                            }
                            Some(val) => val,
                        };

                        Ok(ToDo::Complete(CompleteToDo {
                            to_do_id: existing_id.unwrap(),
                            title,
                            owner: owner_id,
                            description,
                            due_date,
                            completed_on: parsed_completed_on,
                            has_changes: false,
                        }))
                    }
                    _ => Ok(ToDo::Incomplete(IncompleteToDo {
                        to_do_id: id,
                        title,
                        description,
                        due_date,
                        owner: owner_id,
                        has_changes: false,
                    })),
                }
            }
            _ => Ok(ToDo::Incomplete(IncompleteToDo {
                to_do_id: id,
                title,
                description,
                due_date,
                owner: owner_id,
                has_changes: false,
            })),
        }
    }

    /// GET the title of the ToDo
    pub(crate) fn get_title(&self) -> &str {
        match &self {
            ToDo::Incomplete(incomplete) => incomplete.title.to_string(),
            ToDo::Complete(complete) => complete.title.to_string(),
        }
    }

    /// GET the title of the ToDo
    pub(crate) fn get_description(&self) -> &str {
        let desc = match &self {
            ToDo::Incomplete(incomplete) => &incomplete.description,
            ToDo::Complete(complete) => &complete.description,
        };

        match desc {
            None => "",
            Some(val) => val,
        }
    }

    /// GET the title of the ToDo
    pub(crate) fn get_due_date(&self) -> String {
        let due_date = match &self {
            ToDo::Incomplete(incomplete) => incomplete.due_date,
            ToDo::Complete(complete) => complete.due_date,
        };

        match due_date {
            None => String::from(""),
            Some(date) => date.to_rfc3339().to_string(),
        }
    }

    /// GET the date the ToDo was completed. Returns an empty string if incomplete.
    pub(crate) fn get_completed_on(&self) -> String {
        match &self {
            ToDo::Incomplete(_) => String::from(""),
            ToDo::Complete(complete) => complete.completed_on.to_rfc3339().to_string(),
        }
    }

    /// GET the owner of the ToDo
    pub(crate) fn get_owner(&self) -> &str {
        match &self {
            ToDo::Incomplete(incomplete) => incomplete.owner.to_string(),
            ToDo::Complete(complete) => complete.owner.to_string(),
        }
    }

    /// GET the ID of the ToDo
    pub(crate) fn get_id(&self) -> &str {
        match &self {
            ToDo::Incomplete(incomplete) => incomplete.to_do_id.to_string(),
            ToDo::Complete(complete) => complete.to_do_id.to_string(),
        }
    }

    /// GET the status of the ToDo
    pub(crate) fn get_status(&self) -> String {
        match &self {
            ToDo::Incomplete(_) => String::from(INCOMPLETE_STATUS),
            ToDo::Complete(_) => String::from(COMPLETE_STATUS),
        }
    }

    /// Update the title of the existing ToDo.
    /// If the ToDo is already completed then the title cannot be updated.
    /// Returns a new ToDo
    pub(crate) fn update_title(self, new_title: &str) -> Result<ToDo, ValidationError> {
        let new_title_value = Title::new(new_title)?;

        let response = match &self {
            ToDo::Incomplete(incomplete) => ToDo::Incomplete(IncompleteToDo {
                to_do_id: incomplete.to_do_id.clone(),
                title: new_title_value,
                owner: OwnerId::new(incomplete.owner.to_string()).unwrap(),
                description: incomplete.description.clone(),
                due_date: incomplete.due_date,
                has_changes: true,
            }),
            ToDo::Complete(complete) => ToDo::Complete(CompleteToDo {
                to_do_id: complete.to_do_id.clone(),
                title: Title::new(complete.title.to_string()).unwrap(),
                owner: OwnerId::new(complete.owner.to_string()).unwrap(),
                description: complete.description.clone(),
                due_date: complete.due_date,
                completed_on: complete.completed_on,
                has_changes: self.has_changes(),
            }),
        };

        Ok(response)
    }

    pub(crate) fn update_description(self, new_description: Option<String>) -> ToDo {
        let response = match new_description {
            None => self,
            Some(desc) => match &self {
                ToDo::Incomplete(incomplete) => ToDo::Incomplete(IncompleteToDo {
                    to_do_id: incomplete.to_do_id.clone(),
                    title: incomplete.title.clone(),
                    owner: OwnerId::new(incomplete.owner.to_string()).unwrap(),
                    description: Some(desc),
                    due_date: incomplete.due_date,
                    has_changes: true,
                }),
                ToDo::Complete(complete) => ToDo::Complete(CompleteToDo {
                    to_do_id: complete.to_do_id.clone(),
                    title: Title::new(complete.title.to_string()).unwrap(),
                    owner: OwnerId::new(complete.owner.to_string()).unwrap(),
                    description: complete.description.clone(),
                    due_date: complete.due_date,
                    completed_on: complete.completed_on,
                    has_changes: self.has_changes(),
                }),
            },
        };

        response
    }
    pub(crate) fn update_due_date(self, new_due_date: Option<String>) -> ToDo {
        let response = match new_due_date {
            None => self,
            Some(due_date) => {
                let parsed_date = DateTime::parse_from_rfc3339(due_date.as_str());

                match parsed_date {
                    Ok(date) => match &self {
                        ToDo::Incomplete(incomplete) => ToDo::Incomplete(IncompleteToDo {
                            to_do_id: incomplete.to_do_id.clone(),
                            title: incomplete.title.clone(),
                            owner: OwnerId::new(incomplete.owner.to_string()).unwrap(),
                            description: incomplete.description.clone(),
                            due_date: Some(date),
                            has_changes: true,
                        }),
                        ToDo::Complete(complete) => ToDo::Complete(CompleteToDo {
                            to_do_id: complete.to_do_id.clone(),
                            title: Title::new(complete.title.to_string()).unwrap(),
                            owner: OwnerId::new(complete.owner.to_string()).unwrap(),
                            description: complete.description.clone(),
                            due_date: complete.due_date,
                            completed_on: complete.completed_on,
                            has_changes: self.has_changes(),
                        }),
                    },
                    Err(_) => self,
                }
            }
        };

        response
    }

    /// Set the ToDo as completed
    pub(crate) fn set_completed(self) -> ToDo {
        match &self {
            ToDo::Incomplete(incomplete) => ToDo::Complete(CompleteToDo {
                to_do_id: incomplete.to_do_id.clone(),
                title: Title::new(incomplete.title.to_string()).unwrap(),
                owner: OwnerId::new(incomplete.owner.to_string()).unwrap(),
                completed_on: DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap(),
                description: incomplete.description.clone(),
                due_date: incomplete.due_date,
                has_changes: true,
            }),
            ToDo::Complete(complete) => ToDo::Complete(CompleteToDo {
                to_do_id: complete.to_do_id.clone(),
                title: Title::new(complete.title.to_string()).unwrap(),
                owner: OwnerId::new(complete.owner.to_string()).unwrap(),
                description: complete.description.clone(),
                due_date: complete.due_date,
                completed_on: complete.completed_on,
                has_changes: false,
            }),
        }
    }

    fn check_title(input: &Title) -> Result<(), ValidationError> {
        tracing::info!("Checking title: '{}'", input.to_string());

        let _ = check_not_empty_and_length_less_than(input.to_string(), 50)?;

        Ok(())
    }

    fn check_owner_id(input: &OwnerId) -> Result<(), ValidationError> {
        if input.to_string().is_empty() {
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
    description: Option<String>,
    due_date: Option<DateTime<FixedOffset>>,
    owner: OwnerId,
    has_changes: bool,
}

/// Represents the structure of a complete ToDo item
#[non_exhaustive]
pub struct CompleteToDo {
    to_do_id: ToDoId,
    title: Title,
    description: Option<String>,
    due_date: Option<DateTime<FixedOffset>>,
    owner: OwnerId,
    completed_on: DateTime<FixedOffset>,
    has_changes: bool,
}

#[derive(Clone)]
pub(crate) struct ToDoId {
    value: String,
}

impl ToDoId {
    pub fn new() -> ToDoId {
        ToDoId::parse(Uuid::new_v4().to_string().as_str()).unwrap()
    }

    pub fn parse(existing_id: &str) -> Result<ToDoId, ValidationError> {
        let _ = check_not_empty_and_length_less_than(existing_id, 50)?;

        Ok(ToDoId {
            value: existing_id.to_string(),
        })
    }

    pub fn to_string(&self) -> &str {
        self.value.as_str()
    }
}

#[derive(Clone)]
pub(crate) struct Title {
    value: String,
}

impl Title {
    pub fn new(title: &str) -> Result<Title, ValidationError> {
        let _ = check_not_empty_and_length_less_than(title, 50)?;

        Ok(Title {
            value: title.to_string(),
        })
    }

    pub fn to_string(&self) -> &str {
        self.value.as_str()
    }
}

#[derive(Clone)]
pub(crate) struct OwnerId {
    value: String,
}

impl OwnerId {
    pub fn new(owner_id: &str) -> Result<OwnerId, ValidationError> {
        let _ = check_not_empty_and_length_less_than(owner_id, 50)?;

        Ok(OwnerId {
            value: owner_id.to_string(),
        })
    }

    pub fn to_string(&self) -> &str {
        self.value.as_str()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum IsComplete {
    Incomplete,
    Complete,
}

impl fmt::Display for IsComplete {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[async_trait]
pub trait ToDoRepo {
    async fn list(&self, user_id: &str) -> Result<Vec<ToDo>, RepositoryError>;

    async fn create(&self, to_do: &ToDo) -> Result<(), RepositoryError>;

    async fn get(&self, user_id: &str, todo_id: &str) -> Result<ToDo, RepositoryError>;
}

/// Unit tests
///
/// These tests are run using the `cargo test` command.
#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::application::domain::{OwnerId, Title, ToDo};

    use super::ToDoId;

    #[test]
    fn valid_data_should_return_validated_to_do() {
        let to_do = ToDo::new(
            Title::new("my title").unwrap(),
            OwnerId::new("jameseastham").unwrap(),
            Some(String::from("This is the description")),
            None,
        );

        assert!(!to_do.is_err());
        assert_eq!(to_do.as_ref().unwrap().get_title(), "my title");
        assert_eq!(to_do.as_ref().unwrap().get_owner(), "jameseastham");
    }

    #[test]
    fn update_title_for_incomplete_todo_should_change() {
        let todo = ToDo::Incomplete(super::IncompleteToDo {
            to_do_id: ToDoId::parse("hello").unwrap(),
            title: Title::new("hello").unwrap(),
            owner: OwnerId::new("hello").unwrap(),
            description: Some(String::from("This is the description")),
            due_date: None,
            has_changes: false,
        });

        let updated_todo = todo.update_title("my new title");

        if let ToDo::Incomplete(todo) = updated_todo.unwrap() {
            assert_eq!(todo.title.to_string(), String::from("my new title"));
            assert!(todo.has_changes)
        } else {
            panic!("ToDo update method did not return the expected type")
        }
    }

    #[test]
    fn update_title_for_completed_todo_should_not_change() {
        let todo = ToDo::Complete(super::CompleteToDo {
            to_do_id: ToDoId::parse("hello").unwrap(),
            title: Title::new("hello").unwrap(),
            owner: OwnerId::new("hello").unwrap(),
            description: Some(String::from("This is the description")),
            due_date: None,
            completed_on: DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap(),
            has_changes: false,
        });

        let updated_todo = todo.update_title("my new title");

        if let ToDo::Complete(completed) = updated_todo.unwrap() {
            assert_eq!(completed.title.to_string(), String::from("hello"));
            assert!(!completed.has_changes)
        } else {
            panic!("ToDo update method did not return the expected type")
        }
    }

    #[test]
    fn update_status_for_incomplete_todo_should_change() {
        let todo = ToDo::Incomplete(super::IncompleteToDo {
            to_do_id: ToDoId::parse("hello").unwrap(),
            title: Title::new("hello").unwrap(),
            owner: OwnerId::new("hello").unwrap(),
            description: Some(String::from("This is the description")),
            due_date: None,
            has_changes: false,
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
            to_do_id: ToDoId::parse("hello").unwrap(),
            title: Title::new("hello").unwrap(),
            owner: OwnerId::new("hello").unwrap(),
            description: Some(String::from("This is the description")),
            due_date: None,
            completed_on: date,
            has_changes: false,
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
        let option_1 = Some("Hello");
        let option_2: Option<i32> = Some(123456);
        let option_3: Option<i32> = None;

        let valid_res = option_1
            .zip(option_2)
            .map(|(opt1, opt2)| -> String { format!("{opt1} - {opt2}") });

        let none_res = option_1
            .zip(option_3)
            .map(|(opt1, opt2)| -> String { format!("{opt1} - {opt2}") });

        assert_eq!(valid_res, Some("Hello - 123456".to_string()));
        assert_eq!(none_res, None);

        let to_do_id = ToDoId::new();

        assert_eq!(to_do_id.to_string().len(), 36)
    }

    #[test]
    fn parse_empty_id_should_return_validate_error() {
        let to_do_id = ToDoId::parse("");

        assert!(to_do_id.is_err());
    }

    #[test]
    fn empty_title_should_return_validate_error() {
        let to_do = Title::new("");

        assert!(to_do.is_err());
    }

    #[test]
    fn empty_owner_should_return_validate_error() {
        let owner = OwnerId::new("");

        assert!(owner.is_err());
    }
}
