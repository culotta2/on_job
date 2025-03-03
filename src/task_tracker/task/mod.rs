use std::{
    error::Error,
    fmt::Display,
    str::{FromStr, ParseBoolError},
};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Task {
    pub name: String,
    pub tags: Option<Vec<String>>,
    pub complete: bool,
}

impl Task {
    pub fn new(name: String, tags: Option<Vec<String>>) -> Self {
        Task {
            name,
            tags,
            complete: false,
        }
    }

    pub fn complete(&mut self) {
        self.complete = true;
    }
}

#[derive(Debug)]
pub enum TaskError {
    ParseBool(ParseBoolError),
    InvalidTaskFormat,
}

impl Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TaskError::ParseBool(ref e) => e.fmt(f),
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

        if let [name, tags_str, complete_str] = vals[..] {
            // TODO: Check if this check is needed
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
            let complete = complete_str.parse::<bool>()?;
            Ok(Task {
                name: name.into(),
                tags,
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
            "| {} | {} | {} |",
            value.name,
            value.tags.map(|x| x.join(", ")).unwrap_or("".into()),
            value.complete
        )
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            // TODO: Make padding dynamic
            "| {} | {} | {} |",
            self.name,
            self.tags
                .as_ref()
                .map(|x| x.join(", "))
                .unwrap_or("".into()),
            self.complete
        )
    }
}
