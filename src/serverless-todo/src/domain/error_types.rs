use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct RepositoryError {
    error_message: String,
}

impl RepositoryError {
    pub fn new(data_access_error: String) -> RepositoryError {
        RepositoryError {
            error_message: data_access_error,
        }
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

#[derive(Debug, Clone)]
pub struct ValidationError {
    error_message: String,
}

impl ValidationError {
    pub fn new(message: String) -> ValidationError {
        ValidationError {
            error_message: message,
        }
    }

    pub fn to_string(&self) -> String {
        self.error_message.to_string()
    }
}

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Validation error: {0}", self.error_message)
    }
}

impl Error for ValidationError {}
