use serde::{Deserialize, Serialize};

use super::entities::{IsComplete, OwnerId, Title, ToDoId};

#[derive(Deserialize, Serialize)]
pub struct UnvalidatedToDo {
    pub title: String,
    pub owner_id: String,
    pub is_complete: bool,
}

pub struct ValidatedToDo {
    pub id: ToDoId,
    pub title: Title,
    pub is_complete: IsComplete,
    pub owner_id: OwnerId,
}

pub struct CreatedToDo {
    pub id: ToDoId,
    pub title: Title,
    pub is_complete: IsComplete,
    pub owner_id: OwnerId,
}

#[derive(Deserialize, Serialize)]
pub struct ToDoItem {
    pub id: String,
    pub title: String,
    pub is_complete: String,
}
