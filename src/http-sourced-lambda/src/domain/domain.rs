use std::{fmt, error::Error};

use async_trait::async_trait;

use crate::domain::entities::{ToDo, ToDoItem};

#[async_trait]
pub trait Repository {
    async fn store_todo(
        &self,
        body: &ToDo,
    ) -> Result<String, RepositoryError>;

    async fn get_todo(
        &self,
        id: &String,
    ) -> Result<ToDoItem, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct RepositoryError {
    error_message: String
}

impl RepositoryError {
    pub fn new(data_access_error: String) -> RepositoryError {
        RepositoryError { error_message: data_access_error }
    }

    pub fn to_string(&self) -> String {
        // We don't want to disclose the secret
        format!("Error persisiting data {0}", self.error_message)
    }
}

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error persisiting data {0}", self.error_message)
    }
}

impl Error for RepositoryError {}