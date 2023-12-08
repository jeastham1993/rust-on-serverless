use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(crate) enum MessageType {
    ToDoCreated(ToDoCreated),
    ToDoUpdated(ToDoUpdated),
    ToDoCompleted(ToDoCompleted),
}

#[derive(Deserialize, Serialize)]
pub struct ToDoCreated {
    to_do_id: String,
    user_id: String,
}

impl ToDoCreated {
    pub(crate) fn new(to_do_id: String, user_id: String) -> Self {
        Self { to_do_id, user_id }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ToDoCompleted {
    to_do_id: String,
    user_id: String,
}

impl ToDoCompleted {
    pub(crate) fn new(to_do_id: String, user_id: String) -> Self {
        Self { to_do_id, user_id }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ToDoUpdated {
    to_do_id: String,
    user_id: String,
}

impl ToDoUpdated {
    pub(crate) fn new(to_do_id: String, user_id: String) -> Self {
        Self { to_do_id, user_id }
    }
}
