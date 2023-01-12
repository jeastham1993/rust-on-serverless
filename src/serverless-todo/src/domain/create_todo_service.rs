use crate::domain::{
    entities::{Repository, ValidateToDo},
    error_types::ValidationError,
    public_types::{CreatedToDo, UnvalidatedToDo},
};

pub async fn create_to_do(
    input: UnvalidatedToDo,
    client: &dyn Repository,
) -> Result<CreatedToDo, ValidationError> {
    let validation_workflow = ValidateToDo::new(input);

    let to_do = validation_workflow.validate();

    match to_do {
        Ok(val) => {
            let db_res = client.store_todo(val).await;

            match db_res {
                Ok(res) => Ok(res),
                Err(_) => Err(ValidationError::new("Failure creating ToDo".to_string())),
            }
        }
        Err(e) => Err(e),
    }
}
