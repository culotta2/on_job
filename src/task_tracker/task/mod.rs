use chrono::{DateTime, Local, ParseError, Utc};
use std::{
    error::Error,
    fmt::Display,
    str::{FromStr, ParseBoolError},
};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Task {
    pub name: String,
    pub tags: Option<Vec<String>>,
    pub deadline: DateTime<Utc>,
    pub complete: bool,
}

impl Task {
    pub fn new(name: String, tags: Option<Vec<String>>, deadline: DateTime<Utc>) -> Self {
        Task {
            name,
            tags,
            deadline,
            complete: false,
        }
    }

    pub fn complete(&mut self) {
        self.complete = true;
    }

    pub fn local_deadline(&self) -> String {
        self.deadline
            .with_timezone(&Local)
            .format("%m/%d/%Y %H:%M:%S")
            .to_string()
    }

    pub fn export_deadline(&self) -> String {
        self.deadline.to_rfc3339()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseTaskError {
    ParseBool(ParseBoolError),
    InvalidTaskFormat,
    InvalidDateFormat(ParseError),
}

impl Display for ParseTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ParseTaskError::ParseBool(ref e) => e.fmt(f),
            ParseTaskError::InvalidDateFormat(ref e) => e.fmt(f),
            ParseTaskError::InvalidTaskFormat => {
                "provided string could not be converted to a task".fmt(f)
            }
        }
    }
}

impl From<std::str::ParseBoolError> for ParseTaskError {
    fn from(value: std::str::ParseBoolError) -> Self {
        ParseTaskError::ParseBool(value)
    }
}

impl From<ParseError> for ParseTaskError {
    fn from(value: ParseError) -> Self {
        ParseTaskError::InvalidDateFormat(value)
    }
}

impl Error for ParseTaskError {}

impl FromStr for Task {
    type Err = ParseTaskError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vals: Vec<_> = s
            .trim()
            .trim_matches('|')
            .split("|")
            .map(|x| x.trim())
            .collect();

        if let [name, tags_str, complete_str, deadline] = vals[..] {
            let tags = if tags_str
                .split(",")
                .filter(|tag| tag.trim() != "")
                .collect::<Vec<&str>>()
                .is_empty()
            {
                None
            } else {
                Some(
                    tags_str
                        .split(", ")
                        .map(|x| x.into())
                        .collect::<Vec<String>>(),
                )
            };
            let deadline = DateTime::parse_from_rfc3339(deadline.trim())?.to_utc();
            let complete = complete_str.parse::<bool>()?;
            Ok(Task {
                name: name.into(),
                tags,
                deadline,
                complete,
            })
        } else {
            Err(ParseTaskError::InvalidTaskFormat)
        }
    }
}

impl From<Task> for String {
    fn from(value: Task) -> Self {
        format!(
            "| {} | {} | {} | {} |",
            value.name,
            value
                .tags
                .as_ref()
                .map(|x| x.join(", "))
                .unwrap_or("".into()),
            value.complete,
            value.local_deadline()
        )
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "| {} | {} | {} | {} |",
            self.name,
            self.tags
                .as_ref()
                .map(|x| x.join(", "))
                .unwrap_or("".into()),
            self.complete,
            self.export_deadline()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_good_task_single_tag() {
        let task_str = "| Task 1 | ugh | false | 2025-03-17T22:00:00+00:00 |";
        let expected_task = Task {
            name: "Task 1".into(),
            tags: Some(vec!["ugh".into()]),
            deadline: DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
            complete: false,
        };
        let parsed_task = Task::from_str(task_str).unwrap();
        assert_eq!(expected_task, parsed_task)
    }

    #[test]
    fn parse_good_task_multiple_tags() {
        let task_str = "| Task 1 | project, ugh | false | 2025-03-17T22:00:00+00:00 |";
        let expected_task = Task {
            name: "Task 1".into(),
            tags: Some(vec!["project".into(), "ugh".into()]),
            deadline: DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
            complete: false,
        };
        let parsed_task = Task::from_str(task_str).unwrap();
        assert_eq!(expected_task, parsed_task)
    }

    #[test]
    fn parse_good_task_no_tags() {
        let task_str = "| Task 1 | | false | 2025-03-17T22:00:00+00:00 |";
        let expected_task = Task {
            name: "Task 1".into(),
            tags: None,
            deadline: DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
            complete: false,
        };
        let parsed_task = Task::from_str(task_str).unwrap();
        assert_eq!(expected_task, parsed_task)
    }

    #[test]
    #[should_panic]
    fn parse_bad_task_too_few_columns() {
        let task_str = "| My task | Should be deadline |";
        let expected_task = Task {
            name: "Task 1".into(),
            tags: Some(vec!["project".into(), "ugh".into()]),
            deadline: DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
            complete: false,
        };
        let parsed_task = Task::from_str(task_str).unwrap();
        assert_eq!(expected_task, parsed_task)
    }

    #[test]
    #[should_panic]
    fn parse_bad_task_too_many_columns() {
        let task_str = "| project, ugh | false | 2025-03-17T22:00:00+00:00 | Additional stuff";
        let expected_task = Task {
            name: "Task 1".into(),
            tags: Some(vec!["project".into(), "ugh".into()]),
            deadline: DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
            complete: false,
        };
        let parsed_task = Task::from_str(task_str).unwrap();
        assert_eq!(expected_task, parsed_task)
    }

    #[test]
    #[should_panic]
    fn parse_bad_task_no_name() {
        let task_str = "| project, ugh | false | 2025-03-17T22:00:00+00:00 |";
        let expected_task = Task {
            name: "Task 1".into(),
            tags: Some(vec!["project".into(), "ugh".into()]),
            deadline: DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
            complete: false,
        };
        let parsed_task = Task::from_str(task_str).unwrap();
        assert_eq!(expected_task, parsed_task)
    }

    #[test]
    #[should_panic]
    fn parse_bad_task_no_deadline() {
        let task_str = "| project, ugh | 2025-03-17T22:00:00+00:00 |";
        let expected_task = Task {
            name: "Task 1".into(),
            tags: Some(vec!["project".into(), "ugh".into()]),
            deadline: DateTime::parse_from_rfc3339("2025-03-17T22:00:00+00:00")
                .unwrap()
                .to_utc(),
            complete: false,
        };
        let parsed_task = Task::from_str(task_str).unwrap();
        assert_eq!(expected_task, parsed_task)
    }

    #[test]
    #[should_panic]
    fn parse_bad_task_bad_deadline() {
        let task_str = "| project, ugh | 2025-03-40T22:00:00+00:00 |";
        Task::from_str(task_str).unwrap();
    }
}
