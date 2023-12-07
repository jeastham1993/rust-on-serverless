use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ToDoItem {
    pub id: String,
    pub title: String,
    pub is_complete: bool,
    pub completed_on: String,
}

#[derive(Deserialize, Serialize)]
pub struct CreateToDoCommand {
    pub title: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateToDoCommand {
    pub title: String,
    pub set_as_complete: bool,
}