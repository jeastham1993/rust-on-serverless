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
}

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error persisting data {0}", self.error_message)
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

#[derive(Debug, Clone)]
pub struct ServiceError {
    error_message: String,
}

impl ServiceError {
    pub fn new(message: String) -> ServiceError {
        ServiceError {
            error_message: message,
        }
    }
}

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Service error: {0}", self.error_message)
    }
}

impl Error for ServiceError {}
