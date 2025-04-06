use chrono::{DateTime, Utc};

use crate::task_tracker::TaskTracker;
use crate::task_tracker::task::{ParseTaskError, Task};
use crate::utils::right_pad;
use std::collections::HashSet;
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
        all: bool,
        overdue: bool,
        tags: Option<Vec<String>>,
    ) -> Result<(), Self::Err> {
        let file = OpenOptions::new().read(true).open(&self.file_path)?;
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
