pub mod plain_text_task_tracker;
pub mod task;

// TODO: Return actual error types, not string
pub trait TaskTracker {
    type Err;

    fn add_item(&self, name: String, tags: Option<Vec<String>>) -> Result<(), Self::Err>;
    fn complete_item(&mut self, id: usize) -> Result<(), Self::Err>;
    fn delete_item(&mut self, id: usize) -> Result<(), Self::Err>;
    fn list_items(&self) -> Result<(), Self::Err>;
}
