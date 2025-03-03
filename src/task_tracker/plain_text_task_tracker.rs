use crate::task_tracker::task::{Task, TaskError};
use crate::task_tracker::TaskTracker;
use std::error::Error;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub struct PlainTextTaskTracker<'a> {
    file_path: &'a Path,
}

impl<'a> PlainTextTaskTracker<'a> {
    pub fn new(file_path: &'a Path) -> Self {
        PlainTextTaskTracker { file_path }
    }
}

#[derive(Debug)]
pub enum PlainTextTaskTrackerError {
    IO(std::io::Error),
    InvalidTask(TaskError),
}

impl Display for PlainTextTaskTrackerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            PlainTextTaskTrackerError::IO(ref e) => e.fmt(f),
            PlainTextTaskTrackerError::InvalidTask(ref e) => e.fmt(f),
        }
    }
}

impl From<std::io::Error> for PlainTextTaskTrackerError {
    fn from(value: std::io::Error) -> Self {
        PlainTextTaskTrackerError::IO(value)
    }
}

impl From<TaskError> for PlainTextTaskTrackerError {
    fn from(value: TaskError) -> Self {
        PlainTextTaskTrackerError::InvalidTask(value)
    }
}

impl Error for PlainTextTaskTrackerError {}

impl TaskTracker for PlainTextTaskTracker<'_> {
    type Err = PlainTextTaskTrackerError;

    fn add_item(&self, name: String, tags: Option<Vec<String>>) -> Result<(), Self::Err> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.file_path)?;

        let task = Task::new(name, tags);
        let mut writer = BufWriter::new(file);
        writeln!(writer, "{task}")?;
        Ok(())
    }

    fn complete_item(&mut self, id: usize) -> Result<(), Self::Err> {
        let file = OpenOptions::new().read(true).open(self.file_path)?;
        let reader = BufReader::new(file);
        let mut tasks: Vec<Task> = reader
            .lines()
            .map_while(Result::ok)
            .map(|line| line.parse::<Task>())
            .collect::<Result<Vec<Task>, TaskError>>()?;

        tasks
            .iter_mut()
            .rev() // TODO: Change rev to order by deadline desc
            .enumerate()
            .filter(|(idx, _)| *idx == id)
            .for_each(|(_, task)| task.complete());

        let file = OpenOptions::new().write(true).truncate(true).open(self.file_path)?;
        let mut writer = BufWriter::new(file);
        for task in tasks {
            writeln!(writer, "{task}")?;
        }
        Ok(())
    }

    fn delete_item(&mut self, id: usize)-> Result<(), Self::Err> {
        let file = OpenOptions::new().read(true).open(self.file_path)?;
        let reader = BufReader::new(file);
        let tasks: Vec<Task> = reader
            .lines()
            .map_while(Result::ok)
            .map(|line| line.parse::<Task>())
            .collect::<Result<Vec<Task>, TaskError>>()?;

        let tasks: Vec<_> = tasks
            .into_iter()
            .rev()
            .enumerate()
            .filter(|(idx, _) | *idx != id)
            .map(|(_, task)| task)
            .rev()
            .collect()
        ;

        let file = OpenOptions::new().write(true).truncate(true).open(self.file_path)?;
        let mut writer = BufWriter::new(file);
        for task in tasks {
            writeln!(writer, "{task}")?;
        }
        Ok(())
    }
}
