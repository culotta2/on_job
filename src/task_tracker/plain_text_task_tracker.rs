use chrono::{DateTime, Utc};

use crate::task_tracker::TaskTracker;
use crate::task_tracker::task::{ParseTaskError, Task};
use crate::utils::right_pad;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub struct PlainTextTaskTracker<'a> {
    file_path: &'a Path,
}

impl<'a> PlainTextTaskTracker<'a> {
    pub fn new(file_path: &'a Path) -> Self {
        PlainTextTaskTracker { file_path }
    }

    fn read_tasks_from_file(reader: BufReader<File>) -> Result<Vec<Task>, ParseTaskError> {
        let mut tasks: Vec<Task> = reader
            .lines()
            .map_while(Result::ok)
            .map(|line| line.parse::<Task>())
            .collect::<Result<Vec<Task>, ParseTaskError>>()?;
        tasks.sort_by(|task_a, task_b| task_a.deadline.cmp(&task_b.deadline));
        Ok(tasks)
    }
}

#[derive(Debug)]
pub enum PlainTextTaskTrackerError {
    IO(std::io::Error),
    InvalidTask(ParseTaskError),
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

impl From<ParseTaskError> for PlainTextTaskTrackerError {
    fn from(value: ParseTaskError) -> Self {
        PlainTextTaskTrackerError::InvalidTask(value)
    }
}

impl Error for PlainTextTaskTrackerError {}

impl TaskTracker for PlainTextTaskTracker<'_> {
    type Err = PlainTextTaskTrackerError;

    fn add_task(
        &self,
        name: String,
        tags: Option<Vec<String>>,
        deadline: DateTime<Utc>,
    ) -> Result<(), Self::Err> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.file_path)?;
        let task = Task::new(name, tags, deadline);
        let mut writer = BufWriter::new(file);
        writeln!(writer, "{task}")?;
        Ok(())
    }

    fn complete_task(&mut self, id: usize) -> Result<(), Self::Err> {
        let file = OpenOptions::new().read(true).open(self.file_path)?;
        let reader = BufReader::new(file);
        let mut tasks = PlainTextTaskTracker::read_tasks_from_file(reader)?;
        tasks
            .iter_mut()
            .filter(|task| !task.complete)
            .enumerate()
            .filter(|(idx, _)| *idx == id)
            .for_each(|(_, task)| task.complete());

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.file_path)?;
        let mut writer = BufWriter::new(file);
        for task in tasks {
            writeln!(writer, "{task}")?;
        }
        Ok(())
    }

    fn delete_task(&mut self, id: usize) -> Result<(), Self::Err> {
        let file = OpenOptions::new().read(true).open(self.file_path)?;
        let reader = BufReader::new(file);
        let mut tasks = PlainTextTaskTracker::read_tasks_from_file(reader)?;
        if let Some(index) = tasks
            .iter()
            .filter(|task| !task.complete)
            .enumerate()
            .position(|(idx, _)| idx == id)
        {
            tasks.remove(index);
        }
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.file_path)?;
        let mut writer = BufWriter::new(file);
        for task in tasks {
            writeln!(writer, "{task}")?;
        }
        Ok(())
    }

    fn list_task(
        &self,
        all: bool,
        overdue: bool,
        tags: Option<Vec<String>>,
    ) -> Result<(), Self::Err> {
        let file = OpenOptions::new().read(true).open(self.file_path)?;
        let reader = BufReader::new(file);
        let tasks = PlainTextTaskTracker::read_tasks_from_file(reader)?;

        let tasks = match all {
            true => tasks,
            false => tasks.into_iter().filter(|task| !task.complete).collect(),
        };

        let max_idx = tasks.len().to_string().len();
        let (max_name, max_tags) = tasks
            .iter()
            .map(|task| {
                (
                    task.name.len(),
                    task.tags.as_ref().map(|v| 2 * v.len() - 2).unwrap_or(0)
                        + task
                            .tags
                            .as_ref()
                            .map(|v| v.iter().map(|tag| tag.len()).sum())
                            .unwrap_or(1),
                )
            })
            .reduce(|(max_name_len, max_tags_len), (name_len, tags_len)| {
                (
                    std::cmp::max(max_name_len, name_len),
                    std::cmp::max(max_tags_len, tags_len),
                )
            })
            .unwrap_or((1, 1));

        let id_padding = std::cmp::max(max_idx, 1);
        let name_padding = std::cmp::max(max_name, 4);
        let tags_padding = std::cmp::max(max_tags, 4);
        const COMPLETE_PADDING: usize = 6;
        const DEADLINE_PADDING: usize = 19;

        let h_bar = "=".repeat(
            id_padding
                + name_padding
                + tags_padding
                + COMPLETE_PADDING
                + DEADLINE_PADDING
                + (2 * 7),
        );

        let id_field = right_pad("#", id_padding, ' ');
        let name_field = right_pad("Name", name_padding, ' ');
        let tags_field = right_pad("Tags", tags_padding, ' ');
        let complete_field = "Done";
        let deadline_field = right_pad("Due", DEADLINE_PADDING, ' ');

        match all {
            true => {
                println!("| {name_field} | {tags_field} | {deadline_field} | {complete_field} |")
            }
            false => println!(
                "| {id_field} | {name_field} | {tags_field} | {deadline_field} | {complete_field} |"
            ),
        };
        println!("{h_bar}");
        for (idx, task) in tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| match overdue {
                true => (task.deadline <= Utc::now()) & !task.complete,
                false => true,
            })
            .filter(|(_, task)| {
                if let Some(tags) = &tags {
                    task.tags
                        .as_ref()
                        .map(HashSet::from_iter)
                        .unwrap_or(HashSet::new())
                        .intersection(&HashSet::from_iter(tags))
                        .count()
                        > 0
                } else {
                    true
                }
            })
        {
            let id = right_pad(&idx.to_string(), id_padding, ' ');
            let name = right_pad(&task.name, name_padding, ' ');
            let tags = right_pad(
                &task
                    .tags
                    .as_ref()
                    .map(|t| t.join(", "))
                    .unwrap_or(" ".into()),
                tags_padding,
                ' ',
            );
            let deadline = right_pad(&task.local_deadline(), DEADLINE_PADDING, ' ');
            let complete = match task.complete {
                true => "✓",
                false => "",
            };
            match all {
                true => println!("| {name} | {tags} | {deadline} | {complete:^4} |"),
                false => println!("| {id} | {name} | {tags} | {deadline} | {complete:^4} |"),
            }
        }
        Ok(())
    }
}
