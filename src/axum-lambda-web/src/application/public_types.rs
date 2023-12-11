use crate::application::domain::ToDo;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct ToDoItem {
    pub id: String,
    pub title: String,
    pub is_complete: bool,
    pub completed_on: String,
    pub description: String,
    pub due_date: String,
}

impl From<ToDo> for ToDoItem {
    fn from(value: ToDo) -> Self {
        ToDoItem {
            id: value.get_id().to_string(),
            is_complete: match &value {
                ToDo::Incomplete(_) => false,
                ToDo::Complete(_) => true,
            },
            title: value.get_title().to_string(),
            description: value.get_description().to_string(),
            due_date: value.get_due_date(),
            completed_on: value.get_completed_on(),
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
