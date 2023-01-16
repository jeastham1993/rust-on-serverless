use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ToDoItem {
    pub id: String,
    pub title: String,
    pub is_complete: bool,
}

#[derive(Deserialize, Serialize)]
pub struct CreateToDoCommand {
    pub title: String,
    pub owner_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateToDoCommand {
    pub toDoId: String,
    pub title: String,
    pub owner_id: String,
    pub set_as_complete: bool,
}
