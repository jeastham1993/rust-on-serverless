use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(crate) enum MessageType {
    Created(ToDoCreated),
    Updated(ToDoUpdated),
    Completed(ToDoCompleted),
}

#[derive(Deserialize, Serialize)]
pub struct ToDoCreated {
    to_do_id: String,
    user_id: String,
}

impl ToDoCreated {
    pub(crate) fn new(to_do_id: &str, user_id: &str) -> Self {
        Self { to_do_id: to_do_id.to_string(), user_id: user_id.to_string() }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ToDoCompleted {
    to_do_id: String,
    user_id: String,
}

impl ToDoCompleted {
    pub(crate) fn new(to_do_id: &str, user_id: &str) -> Self {
        Self { to_do_id: to_do_id.to_string(), user_id: user_id.to_string() }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ToDoUpdated {
    to_do_id: String,
    user_id: String,
}

impl ToDoUpdated {
    pub(crate) fn new(to_do_id: &str, user_id: &str) -> Self {
        Self { to_do_id: to_do_id.to_string(), user_id: user_id.to_string() }
    }
}
