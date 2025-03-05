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
    pub deadline: Option<DateTime<Utc>>,
    pub complete: bool,
}

impl Task {
    pub fn new(name: String, tags: Option<Vec<String>>, deadline: Option<DateTime<Utc>>) -> Self {
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
            .map(|dt| {
                dt.with_timezone(&Local)
                    .format("%m/%d/%Y %H:%m:%S")
                    .to_string()
            })
            .unwrap_or("".into())
    }

    pub fn export_deadline(&self) -> String {
        self.deadline.map(|dt| dt.to_rfc3339()).unwrap_or("".into())
    }
}

#[derive(Debug)]
pub enum TaskError {
    ParseBool(ParseBoolError),
    InvalidTaskFormat,
    InvalidDateFormat(ParseError),
}

impl Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TaskError::ParseBool(ref e) => e.fmt(f),
            TaskError::InvalidDateFormat(ref e) => e.fmt(f),
            TaskError::InvalidTaskFormat => {
                "provided string could not be converted to a task".fmt(f)
            }
        }
    }
}

impl From<std::str::ParseBoolError> for TaskError {
    fn from(value: std::str::ParseBoolError) -> Self {
        TaskError::ParseBool(value)
    }
}

impl From<ParseError> for TaskError {
    fn from(value: ParseError) -> Self {
        TaskError::InvalidDateFormat(value)
    }
}

impl Error for TaskError {}

impl FromStr for Task {
    type Err = TaskError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vals: Vec<_> = s
            .trim()
            .trim_matches('|')
            .split("|")
            .map(|x| x.trim())
            .collect();

        if let [name, tags_str, complete_str, deadline] = vals[..] {
            let tags = if tags_str.split(",").collect::<Vec<&str>>().is_empty() {
                None
            } else {
                Some(
                    tags_str
                        .split(", ")
                        .map(|x| x.into())
                        .collect::<Vec<String>>(),
                )
            };
            let deadline = if deadline.trim().is_empty() {
                None
            } else {
                Some(DateTime::parse_from_rfc3339(deadline.trim())?.to_utc())
            };
            let complete = complete_str.parse::<bool>()?;
            Ok(Task {
                name: name.into(),
                tags,
                deadline,
                complete,
            })
        } else {
            Err(TaskError::InvalidTaskFormat)
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
