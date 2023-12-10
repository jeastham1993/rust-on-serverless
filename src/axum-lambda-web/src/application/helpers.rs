use crate::application::error_types::ValidationError;

pub fn check_not_empty_and_length_less_than(input: &str, max_len: i64) -> Result<(), ValidationError> {
    if input.is_empty() || input.len() > 50 {
        Err(ValidationError::new(
            format!("Must be between 1 and {} chars", max_len),
        ))
    }
    else {
        Ok(())
    }
}