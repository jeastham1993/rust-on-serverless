use crate::domain::{
    entities::{Repository},
    error_types::ValidationError,
};

use super::{public_types::{CreateToDoCommand, ToDoItem}, entities::{ToDo, Title, OwnerId}};

pub async fn create_to_do(
    input: CreateToDoCommand,
    client: &dyn Repository,
) -> Result<ToDoItem, ValidationError> {
    let to_do = ToDo::new(Title::Title(input.title), OwnerId::OwnerId(input.owner_id), Option::None, Option::None);

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
        },
    }
}
