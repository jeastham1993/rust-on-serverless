use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    error_types::{RepositoryError, ValidationError},
    public_types::{CreatedToDo, ToDoItem, UnvalidatedToDo, ValidatedToDo},
};

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

pub struct ValidateToDo {
    title: Option<Title>,
    owner_id: Option<OwnerId>,
    is_complete: IsComplete,
    pub errors: Vec<ValidationError>,
    to_validate: UnvalidatedToDo,
}

impl ValidateToDo {
    pub fn new(unvalidated_todo: UnvalidatedToDo) -> Self {
        ValidateToDo {
            title: Option::None,
            owner_id: Option::None,
            is_complete: IsComplete::INCOMPLETE,
            errors: Vec::new(),
            to_validate: unvalidated_todo,
        }
    }

    pub fn validate(mut self) -> Result<ValidatedToDo, ValidationError> {
        self = self.check_title().check_owner_id();

        if self.errors.len() > 0 {
            let mut errors = "".to_string();

            for ele in &self.errors {
                let message = format!("{} - {}", errors, ele.to_string()).to_string();

                errors = message.clone();
            }

            return Err(ValidationError::new(errors.to_string()));
        }

        Ok(ValidatedToDo {
            id: ToDoId::ToDoId(Uuid::new_v4().to_string()),
            title: self.title.unwrap(),
            is_complete: self.is_complete,
            owner_id: self.owner_id.unwrap(),
        })
    }

    fn check_title(mut self) -> Self {
        let input = self.to_validate.title.clone();

        if input.len() > 0 && input.len() <= 50 {
            self.title = Some(Title::Title(input));
        } else {
            self.errors.push(ValidationError::new(
                "Must be between 1 and 50 chars".to_string(),
            ))
        }

        self
    }

    fn check_owner_id(mut self) -> Self {
        let input = self.to_validate.owner_id.clone();
        if input.len() > 0 {
            self.owner_id = Some(OwnerId::OwnerId(input))
        } else {
            self.errors.push(ValidationError::new(
                "Owner Id must have a length".to_string(),
            ));
        }

        self
    }
}

#[async_trait]
pub trait Repository {
    async fn store_todo(&self, body: ValidatedToDo) -> Result<CreatedToDo, RepositoryError>;

    async fn get_todo(&self, id: &String) -> Result<ToDoItem, RepositoryError>;
}

/// Unit tests
///
/// These tests are run using the `cargo test` command.
#[cfg(test)]
mod tests {
    use crate::domain::public_types::UnvalidatedToDo;

    use super::ValidateToDo;

    #[test]
    fn valid_data_should_return_validated_to_do() {
        let validator = ValidateToDo::new(UnvalidatedToDo {
            is_complete: false,
            owner_id: "jameseastham".to_string(),
            title: "my title".to_string(),
        });

        let to_do = validator.validate();

        let res = to_do.as_ref().unwrap();

        assert_eq!(to_do.is_err(), false);
        assert_eq!(res.title.to_string(), "my title");
        assert_eq!(res.owner_id.to_string(), "jameseastham");
        assert_eq!(res.is_complete.to_string(), "INCOMPLETE");
    }

    #[test]
    fn empty_title_should_return_validate_error() {
        let validator = ValidateToDo::new(UnvalidatedToDo {
            is_complete: false,
            owner_id: "jameseastham".to_string(),
            title: "".to_string(),
        });

        let res = validator.validate();

        assert_eq!(res.is_err(), true);
        assert_eq!(
            res.err().unwrap().to_string(),
            "Validation error:  - Validation error: Must be between 1 and 50 chars"
        );
    }

    #[test]
    fn empty_owner_should_return_validate_error() {
        let validator = ValidateToDo::new(UnvalidatedToDo {
            is_complete: false,
            owner_id: "".to_string(),
            title: "my title".to_string(),
        });

        let res = validator.validate();

        assert_eq!(res.is_err(), true);
        assert_eq!(
            res.err().unwrap().to_string(),
            "Validation error:  - Validation error: Owner Id must have a length"
        );
    }
}
