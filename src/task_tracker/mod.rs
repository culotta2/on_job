use chrono::{DateTime, Utc};

pub mod plain_text_task_tracker;
pub mod task;

pub trait TaskTracker {
    type Err;

    fn add_task(
        &self,
        name: String,
        tags: Option<Vec<String>>,
        deadline: DateTime<Utc>,
    ) -> Result<(), Self::Err>;
    fn complete_task(&mut self, id: usize) -> Result<(), Self::Err>;
    fn delete_task(&mut self, id: usize) -> Result<(), Self::Err>;
    fn list_task(
        &self,
        incomplete: bool,
        overdue: bool,
        tags: Option<Vec<String>>,
    ) -> Result<(), Self::Err>;
}
