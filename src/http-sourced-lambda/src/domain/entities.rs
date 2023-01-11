use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct ToDoId {
    value: String
}

impl ToDoId {
    pub fn new(value: String) -> Result<Self, ()> {
        if value.len() == 0 {
            Ok(ToDoId{
                value
            })
        } else {
            Err(())
        }
    }

    pub fn value(&self) -> &String {
        &self.value
    }
}

pub struct Title {
    value: String
}

impl Title {
    pub fn new(value: String) -> Result<Self, ()> {
        if value.len() == 0 {
            Ok(Title{
                value
            })
        } else {
            Err(())
        }
    }

    pub fn value(&self) -> &String {
        &self.value
    }
}

#[derive(Debug)]
pub enum IsComplete  {
    INCOMPLETE,
    COMPLETE,
}

impl fmt::Display for IsComplete {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct OwnerId {
    value: String
}

impl OwnerId {
    pub fn new(value: String) -> Result<Self, ()> {
        if value.len() == 0 {
            Ok(OwnerId{
                value
            })
        } else {
            Err(())
        }
    }

    pub fn value(&self) -> &String {
        &self.value
    }
}

pub struct ToDo {
    pub id: ToDoId,
    pub title: Title,
    pub is_complete: IsComplete,
    pub owner_id: OwnerId
}

impl ToDo {
    pub fn new(title: String, is_complete: IsComplete, owner_id: String) -> ToDo{
        ToDo {
            id: ToDoId::new(Uuid::new_v4().to_string()).unwrap(),
            title: Title::new(title).expect("Invalid title provided"),
            is_complete: is_complete,
            owner_id: OwnerId::new(owner_id).expect("Invalid owner Id"),
        }
    }

    pub fn as_to_do_item(&self) -> ToDoItem {
        ToDoItem {
            id: self.id.value().to_string(),
            title: self.title.value().to_string(),
            is_complete: self.is_complete.to_string()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ToDoItem {
    pub id: String,
    pub title: String,
    pub is_complete: String
}