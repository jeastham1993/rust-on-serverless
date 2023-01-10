use uuid::Uuid;

use super::dto::ToDoItem;

pub struct ToDo {
    id: String,
    title: String,
    is_complete: bool,
    owner_id: String
}

impl ToDo {
    pub fn new(title: String, is_complete: bool, owner_id: String) -> ToDo{
        ToDo {
            id: Uuid::new_v4().to_string(),
            title: title,
            is_complete: is_complete,
            owner_id: owner_id,
        }
    }

    pub fn as_to_do_item(&self) -> ToDoItem {
        ToDoItem {
            id: self.id.clone(),
            title: self.title.clone(),
            is_complete: self.is_complete
        }
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn get_is_complete(&self) -> bool {
        self.is_complete
    }

    pub fn get_owner_id(&self) -> &String {
        &self.owner_id
    }
}