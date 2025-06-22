use chrono::{DateTime, Utc};

use crate::task_tracker::TaskTracker;
use crate::task_tracker::task::{ParseTaskError, Task};
use crate::utils::{TextEffect, add_text_effect};
use std::error::Error;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

pub struct PlainTextTaskTracker {
    file_path: PathBuf,
}

impl PlainTextTaskTracker {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        PlainTextTaskTracker {
            file_path: file_path.into(),
        }
    }

    fn read_tasks_from_file<B: BufRead>(reader: B) -> Result<Vec<Task>, ParseTaskError> {
        let mut tasks: Vec<Task> = reader
            .lines()
            .map_while(Result::ok)
            .map(|line| line.parse::<Task>())
            .collect::<Result<Vec<Task>, ParseTaskError>>()?;
        tasks.sort_by(|task_a, task_b| task_a.deadline.cmp(&task_b.deadline));
        Ok(tasks)
    }

    fn write_tasks_to_file<W: Write>(writer: &mut W, tasks: &[Task]) -> Result<(), std::io::Error> {
        for task in tasks {
            writeln!(writer, "{task}")?
        }
        Ok(())
    }

    fn add_task_logic<W: Write>(writer: &mut W, task: Task) -> Result<(), std::io::Error> {
        writeln!(writer, "{task}")?;
        Ok(())
    }

    fn complete_task_logic(tasks: &mut [Task], id: usize) {
        tasks
            .iter_mut()
            .filter(|task| !task.complete)
            .enumerate()
            .filter(|(idx, _)| *idx == id)
            .for_each(|(_, task)| task.complete());
    }

    fn delete_task_logic(tasks: &mut Vec<Task>, id: usize) {
        if let Some((_, delete_idx)) = tasks
            .iter()
            .scan((None, None), |(incomplete_idx, total_idx), task| {
                *total_idx = (*total_idx).map(|idx| idx + 1).or(Some(0));
                if !task.complete {
                    *incomplete_idx = (*incomplete_idx).map(|idx| idx + 1).or(Some(0));
                }
                Some((*incomplete_idx, *total_idx))
            })
            .flat_map(|(incomplete_idx, total_idx)| Some((incomplete_idx?, total_idx?)))
            .find(|(incomplete_idx, _)| *incomplete_idx == id)
        {
            _ = tasks.remove(delete_idx);
        }
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

impl TaskTracker for PlainTextTaskTracker {
    type Err = PlainTextTaskTrackerError;

    fn add_task(
        &self,
        name: String,
        tags: Option<Vec<String>>,
        deadline: DateTime<Utc>,
    ) -> Result<(), Self::Err> {
        let task = Task::new(name, tags, deadline);
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.file_path)?;
        let mut writer = BufWriter::new(file);
        PlainTextTaskTracker::add_task_logic(&mut writer, task)?;
        Ok(())
    }

    fn complete_task(&mut self, id: usize) -> Result<(), Self::Err> {
        let file = OpenOptions::new().read(true).open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut tasks = PlainTextTaskTracker::read_tasks_from_file(reader)?;
        PlainTextTaskTracker::complete_task_logic(&mut tasks, id);

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.file_path)?;
        let mut writer = BufWriter::new(file);
        PlainTextTaskTracker::write_tasks_to_file(&mut writer, &tasks)?;

        Ok(())
    }

    fn delete_task(&mut self, id: usize) -> Result<(), Self::Err> {
        let file = OpenOptions::new().read(true).open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut tasks = PlainTextTaskTracker::read_tasks_from_file(reader)?;
        PlainTextTaskTracker::delete_task_logic(&mut tasks, id);

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.file_path)?;
        let mut writer = BufWriter::new(file);
        PlainTextTaskTracker::write_tasks_to_file(&mut writer, &tasks)?;

        Ok(())
    }

    fn list_task(
        &self,
        incomplete: bool,
        overdue: bool,
        tags: Option<Vec<String>>,
    ) -> Result<(), Self::Err> {
        let file = OpenOptions::new().read(true).open(&self.file_path)?;
        let reader = BufReader::new(file);
        let tasks = PlainTextTaskTracker::read_tasks_from_file(reader)?;

        let tasks = match incomplete {
            true => tasks,
            false => tasks.into_iter().filter(|task| !task.complete).collect(),
        };

        const COLUMN_PADDING: usize = 4;
        const DEADLINE_LENGTH: usize = 16;
        let max_id_length = (1 + tasks.len().ilog10() as usize).max(2);
        let mut max_name_length = 2;
        let mut max_tags_length = 4;

        for task in &tasks {
            max_name_length = max_name_length.max(task.name.len());
            max_tags_length =
                max_tags_length.max(task.tags.as_deref().unwrap_or_default().join(", ").len());
        }

        if incomplete {
            println!(
                "Name{}Tags{}Due",
                " ".repeat(max_name_length - "Name".len() + COLUMN_PADDING),
                " ".repeat(max_tags_length - "Tags".len() + COLUMN_PADDING),
            );

            println!(
                "{}",
                "-".repeat(
                    max_name_length + max_tags_length + DEADLINE_LENGTH + (COLUMN_PADDING * 2)
                )
            );
            for task in tasks.iter() {
                if overdue && task.deadline.gt(&Utc::now()) {
                    continue;
                }
                if let Some(tags) = &tags {
                    if !tags
                        .iter()
                        .all(|tag| task.tags.as_deref().unwrap_or_default().contains(tag))
                    {
                        continue;
                    }
                }

                let deadline_str = task.local_deadline();
                let tags_display = task.tags.as_deref().unwrap_or_default().join(", ");
                match (&task.complete, &task.deadline.lt(&Utc::now())) {
                    (&true, _) => {
                        println!(
                            "{}{}{}{}{}",
                            add_text_effect(
                                &add_text_effect(&task.name, TextEffect::StrikeThrough),
                                TextEffect::Green
                            ),
                            " ".repeat(max_name_length - task.name.len() + COLUMN_PADDING),
                            add_text_effect(
                                &add_text_effect(&tags_display, TextEffect::StrikeThrough),
                                TextEffect::Green
                            ),
                            " ".repeat(max_tags_length - tags_display.len() + COLUMN_PADDING),
                            add_text_effect(
                                &add_text_effect(&deadline_str, TextEffect::StrikeThrough),
                                TextEffect::Green
                            )
                        );
                    }
                    (&false, &true) => {
                        println!(
                            "{}{}{}{}{}",
                            add_text_effect(&task.name, TextEffect::Red),
                            " ".repeat(max_name_length - task.name.len() + COLUMN_PADDING),
                            add_text_effect(&tags_display, TextEffect::Red),
                            " ".repeat(max_tags_length - tags_display.len() + COLUMN_PADDING),
                            add_text_effect(&deadline_str, TextEffect::Red)
                        );
                    }
                    (&false, &false) => {
                        println!(
                            "{}{}{}{}{}",
                            &task.name,
                            " ".repeat(max_name_length - task.name.len() + COLUMN_PADDING),
                            &tags_display,
                            " ".repeat(max_tags_length - tags_display.len() + COLUMN_PADDING),
                            &deadline_str,
                        );
                    }
                }
            }
        } else {
            println!(
                "Id{}Name{}Tags{}Due",
                " ".repeat(max_id_length - "Id".len() + COLUMN_PADDING),
                " ".repeat(max_name_length - "Name".len() + COLUMN_PADDING),
                " ".repeat(max_tags_length - "Tags".len() + COLUMN_PADDING),
            );

            println!(
                "{}",
                "-".repeat(
                    max_name_length
                        + max_tags_length
                        + max_id_length
                        + DEADLINE_LENGTH
                        + (COLUMN_PADDING * 3)
                )
            );

            for (id, task) in tasks.iter().enumerate() {
                if overdue && task.deadline.gt(&Utc::now()) {
                    continue;
                }

                if let Some(tags) = &tags {
                    if !tags
                        .iter()
                        .all(|tag| task.tags.as_deref().unwrap_or_default().contains(tag))
                    {
                        continue;
                    }
                }

                let deadline_str = task.local_deadline();
                let tags_display = task.tags.as_deref().unwrap_or_default().join(", ");
                match &task.deadline.lt(&Utc::now()) {
                    true => {
                        println!(
                            "{}{}{}{}{}{}{}",
                            add_text_effect(&id.to_string(), TextEffect::Red),
                            " ".repeat(
                                max_id_length - (id.checked_ilog10().unwrap_or(0) as usize + 1)
                                    + COLUMN_PADDING
                            ),
                            add_text_effect(&task.name, TextEffect::Red),
                            " ".repeat(max_name_length - task.name.len() + COLUMN_PADDING),
                            add_text_effect(&tags_display, TextEffect::Red),
                            " ".repeat(max_tags_length - tags_display.len() + COLUMN_PADDING),
                            add_text_effect(&deadline_str, TextEffect::Red),
                        );
                    }
                    false => {
                        println!(
                            "{}{}{}{}{}{}{}",
                            &id.to_string(),
                            " ".repeat(
                                max_id_length - (id.checked_ilog10().unwrap_or(0) as usize + 1)
                                    + COLUMN_PADDING
                            ),
                            &task.name,
                            " ".repeat(max_name_length - task.name.len() + COLUMN_PADDING),
                            &tags_display,
                            " ".repeat(max_tags_length - tags_display.len() + COLUMN_PADDING),
                            deadline_str
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::DateTime;
    use std::io::Cursor;

    #[test]
    fn plain_text_task_tracker_parse_file_one_task() {
        let data = "| Task 1 | ugh | false | 2025-03-17T22:00:00+00:00 |\n";
        let reader = BufReader::new(Cursor::new(data.as_bytes()));
        let actual_task = PlainTextTaskTracker::read_tasks_from_file(reader);
        let expected_task = Ok(vec![Task {
            name: "Task 1".into(),
            tags: Some(vec!["ugh".into()]),
            deadline: DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
            complete: false,
        }]);
        assert_eq!(expected_task, actual_task);
    }

    #[test]
    fn plain_text_task_tracker_parse_file_multiple_tasks() {
        let data = "| Task 1 | ugh | false | 2025-03-17T22:00:00+00:00 |\n| Task 2 | project, time | true | 2025-03-19T22:00:00+00:00 |";
        let reader = BufReader::new(Cursor::new(data.as_bytes()));
        let actual_task = PlainTextTaskTracker::read_tasks_from_file(reader);
        let expected_task = Ok(vec![
            Task {
                name: "Task 1".into(),
                tags: Some(vec!["ugh".into()]),
                deadline: DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
                complete: false,
            },
            Task {
                name: "Task 2".into(),
                tags: Some(vec!["project".into(), "time".into()]),
                deadline: DateTime::parse_from_rfc3339("2025-03-19T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
                complete: true,
            },
        ]);
        assert_eq!(expected_task, actual_task);
    }

    #[test]
    fn plain_text_task_tracker_sorting_unique_deadlines() {
        let data = br#"| Task 1 | project, ugh | false | 2025-03-01T22:00:00+00:00 |
| Task 2 | project, ugh | false | 2025-03-02T13:01:00+00:00 |
| Task 3 | project, ugh | false | 2025-03-02T13:00:00+00:00 |
| Task 4 | project, ugh | false | 2025-01-10T16:00:00+00:00 |
| Task 5 | project, ugh | true | 2025-02-20T03:00:00+00:00 |"#;
        let reader = BufReader::new(Cursor::new(data));
        let actual_tasks = PlainTextTaskTracker::read_tasks_from_file(reader);
        assert!(actual_tasks.is_ok());
        let expected_tasks = vec![
            Task {
                name: "Task 4".into(),
                tags: Some(vec!["project".into(), "ugh".into()]),
                complete: false,
                deadline: DateTime::parse_from_rfc3339("2025-01-10T16:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            Task {
                name: "Task 5".into(),
                tags: Some(vec!["project".into(), "ugh".into()]),
                complete: true,
                deadline: DateTime::parse_from_rfc3339("2025-02-20T03:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            Task {
                name: "Task 1".into(),
                tags: Some(vec!["project".into(), "ugh".into()]),
                complete: false,
                deadline: DateTime::parse_from_rfc3339("2025-03-01T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            Task {
                name: "Task 3".into(),
                tags: Some(vec!["project".into(), "ugh".into()]),
                complete: false,
                deadline: DateTime::parse_from_rfc3339("2025-03-02T13:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            Task {
                name: "Task 2".into(),
                tags: Some(vec!["project".into(), "ugh".into()]),
                complete: false,
                deadline: DateTime::parse_from_rfc3339("2025-03-02T13:01:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
        ];
        assert_eq!(expected_tasks, actual_tasks.unwrap());
    }

    #[test]
    fn plain_text_task_tracker_sorting_non_unique_deadlines() {
        let data = br#"| Task 1 | project, ugh | false | 2025-03-01T22:00:00+00:00 |
| Task 2 | project, ugh | false | 2025-03-02T13:00:00+00:00 |
| Task 3 | project, ugh | false | 2025-03-02T13:00:00+00:00 |
| Task 4 | project, ugh | false | 2025-03-01T22:00:00+00:00 |
| Task 5 | project, ugh | true | 2025-02-20T03:00:00+00:00 |"#;
        let reader = BufReader::new(Cursor::new(data));
        let actual_tasks = PlainTextTaskTracker::read_tasks_from_file(reader);
        assert!(actual_tasks.is_ok());
        let expected_tasks = vec![
            Task {
                name: "Task 5".into(),
                tags: Some(vec!["project".into(), "ugh".into()]),
                complete: true,
                deadline: DateTime::parse_from_rfc3339("2025-02-20T03:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            Task {
                name: "Task 1".into(),
                tags: Some(vec!["project".into(), "ugh".into()]),
                complete: false,
                deadline: DateTime::parse_from_rfc3339("2025-03-01T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            Task {
                name: "Task 4".into(),
                tags: Some(vec!["project".into(), "ugh".into()]),
                complete: false,
                deadline: DateTime::parse_from_rfc3339("2025-03-01T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            Task {
                name: "Task 2".into(),
                tags: Some(vec!["project".into(), "ugh".into()]),
                complete: false,
                deadline: DateTime::parse_from_rfc3339("2025-03-02T13:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            Task {
                name: "Task 3".into(),
                tags: Some(vec!["project".into(), "ugh".into()]),
                complete: false,
                deadline: DateTime::parse_from_rfc3339("2025-03-02T13:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
        ];
        assert_eq!(expected_tasks, actual_tasks.unwrap());
    }

    #[test]
    fn plain_text_task_tracker_write_single_task() {
        let data = b"";
        let mut cursor = Cursor::new(data.to_vec());

        let new_task = Task::new(
            "Task 3".into(),
            Some(vec!["workin'".into()]),
            DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
        );

        let res = PlainTextTaskTracker::write_tasks_to_file(&mut cursor, &[new_task]);
        assert!(res.is_ok());

        let actual_output = cursor.get_ref();

        let expected_output = b"| Task 3 | workin' | false | 2025-03-17T22:00:00+00:00 |\n";
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn plain_text_task_tracker_write_multiple_tasks() {
        let data = b"";
        let mut cursor = Cursor::new(data.to_vec());

        let tasks = [
            Task::new(
                "Task 1".into(),
                None,
                DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
            Task::new(
                "Task 2".into(),
                Some(vec!["sleepin'".into()]),
                DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
            Task::new(
                "Task 3".into(),
                Some(vec!["workin'".into()]),
                DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
        ];

        let res = PlainTextTaskTracker::write_tasks_to_file(&mut cursor, &tasks);
        assert!(res.is_ok());

        let actual_output = cursor.get_ref();

        let expected_output = br#"| Task 1 |  | false | 2025-03-17T22:00:00+00:00 |
| Task 2 | sleepin' | false | 2025-03-17T22:00:00+00:00 |
| Task 3 | workin' | false | 2025-03-17T22:00:00+00:00 |
"#;
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn plain_text_task_tracker_add_task_to_empty_file() {
        let data = b"";
        let mut cursor = Cursor::new(data.to_vec());

        let new_task = Task::new(
            "Task 3".into(),
            Some(vec!["workin'".into()]),
            DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
        );

        let res = PlainTextTaskTracker::add_task_logic(&mut cursor, new_task);
        assert!(res.is_ok());

        let actual_output = cursor.get_ref();

        let expected_output = b"| Task 3 | workin' | false | 2025-03-17T22:00:00+00:00 |\n";
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn plain_text_task_tracker_add_task_to_populated_file() {
        let data = b"| Task 1 | ugh | false | 2025-03-17T22:00:00+00:00 |\n| Task 2 | project, time | true | 2025-03-19T22:00:00+00:00 |\n";
        let mut cursor = Cursor::new(data.to_vec());
        cursor.set_position(data.len() as u64);

        let new_task = Task::new(
            "Task 3".into(),
            Some(vec!["workin'".into()]),
            DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
        );

        let res = PlainTextTaskTracker::add_task_logic(&mut cursor, new_task);
        assert!(res.is_ok());

        let actual_output = cursor.get_ref();

        let expected_output = b"| Task 1 | ugh | false | 2025-03-17T22:00:00+00:00 |\n| Task 2 | project, time | true | 2025-03-19T22:00:00+00:00 |\n| Task 3 | workin' | false | 2025-03-17T22:00:00+00:00 |\n";
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn plain_text_task_tracker_complete_task_id_exists() {
        let mut tasks = vec![
            Task::new(
                "Task 0".into(),
                Some(vec!["ugh".into(), "project".into()]),
                DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
            Task::new(
                "Task 1".into(),
                None,
                DateTime::parse_from_rfc3339("2025-03-18T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
        ];
        let id = 1;
        PlainTextTaskTracker::complete_task_logic(&mut tasks, id);

        let expected_tasks = [
            Task::new(
                "Task 0".into(),
                Some(vec!["ugh".into(), "project".into()]),
                DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
            Task {
                name: "Task 1".into(),
                tags: None,
                deadline: DateTime::parse_from_rfc3339("2025-03-18T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
                complete: true,
            },
        ];

        assert_eq!(tasks, expected_tasks)
    }

    #[test]
    fn plain_text_task_tracker_complete_task_id_does_not_exist() {
        let mut tasks = vec![
            Task::new(
                "Task 0".into(),
                Some(vec!["ugh".into(), "project".into()]),
                DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
            Task::new(
                "Task 1".into(),
                None,
                DateTime::parse_from_rfc3339("2025-03-18T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
        ];
        let id = 100;
        PlainTextTaskTracker::complete_task_logic(&mut tasks, id);

        let expected_tasks = [
            Task::new(
                "Task 0".into(),
                Some(vec!["ugh".into(), "project".into()]),
                DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
            Task::new(
                "Task 1".into(),
                None,
                DateTime::parse_from_rfc3339("2025-03-18T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
        ];

        assert_eq!(tasks, expected_tasks)
    }

    #[test]
    fn plain_text_task_tracker_delete_task_id_exists() {
        let mut tasks = vec![
            Task::new(
                "Task 0".into(),
                Some(vec!["ugh".into(), "project".into()]),
                DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
            Task::new(
                "Task 1".into(),
                None,
                DateTime::parse_from_rfc3339("2025-03-18T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
        ];
        let id = 1;
        PlainTextTaskTracker::delete_task_logic(&mut tasks, id);

        let expected_tasks = [Task::new(
            "Task 0".into(),
            Some(vec!["ugh".into(), "project".into()]),
            DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
        )];

        assert_eq!(tasks, expected_tasks)
    }

    #[test]
    fn plain_text_task_tracker_delete_task_id_does_not_exist() {
        let mut tasks = vec![
            Task::new(
                "Task 0".into(),
                Some(vec!["ugh".into(), "project".into()]),
                DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
            Task::new(
                "Task 1".into(),
                None,
                DateTime::parse_from_rfc3339("2025-03-18T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
        ];
        let id = 100;
        PlainTextTaskTracker::delete_task_logic(&mut tasks, id);

        let expected_tasks = [
            Task::new(
                "Task 0".into(),
                Some(vec!["ugh".into(), "project".into()]),
                DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
            Task::new(
                "Task 1".into(),
                None,
                DateTime::parse_from_rfc3339("2025-03-18T22:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            ),
        ];

        assert_eq!(tasks, expected_tasks)
    }
}
