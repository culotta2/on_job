pub mod plain_text_task_tracker;
pub mod task;

// TODO: Return actual error types, not string
pub trait TaskTracker {
    type Err;

    fn add_task(&self, name: String, tags: Option<Vec<String>>) -> Result<(), Self::Err>;
    fn complete_task(&mut self, id: usize) -> Result<(), Self::Err>;
    fn delete_task(&mut self, id: usize) -> Result<(), Self::Err>;
    fn list_task(&self) -> Result<(), Self::Err>;
}
