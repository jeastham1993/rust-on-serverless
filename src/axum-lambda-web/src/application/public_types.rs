use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ToDoItem {
    pub id: String,
    pub title: String,
    pub is_complete: bool,
    pub completed_on: String,
    pub description: String,
    pub due_date: String
}

impl ToDoItem {
    pub fn empty() -> Self {
        ToDoItem {
            id: String::from(""),
            title: String::from(""),
            is_complete: false,
            completed_on: String::from(""),
            description: String::from(""),
            due_date:String::from(""),
        }
    }
}
#[derive(Deserialize, Serialize)]
pub struct CreateToDoCommand {
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateToDoCommand {
    pub title: String,
    pub set_as_complete: bool,
    pub description: Option<String>,
    pub due_date: Option<String>,
}
