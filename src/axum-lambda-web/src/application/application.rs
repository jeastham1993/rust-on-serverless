use std::fmt::Error;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub todo_repo: Arc<dyn ToDoRepo + Send + Sync>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTodo {
    pub text: String,
}

#[async_trait]
pub trait ToDoRepo
{
    async fn list(&self, user_id: &str) -> Result<Vec<Todo>, Error>;

    async fn create(&self, to_do: Todo) -> Result<(), Error>;

    async fn get (&self, todo_id: &str) -> Result<Todo, Error>;
}

#[derive(Debug, Serialize, Clone)]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub completed: bool,
}

pub async fn create_todo(){

}