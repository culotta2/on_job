use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone};
use clap::Parser;
use std::{fmt::Display, path::PathBuf, str::FromStr};
use task_tracker::TaskTracker;
mod task_tracker;
mod utils;

#[derive(Debug, Parser)]
#[command(
    author="Dominic Culotta",
    version="0.3.0",
    about="A todo CLI application",
    long_about = None
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    file: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct AddTaskArgs {
    #[arg(short, long)]
    /// Name of project
    name: String,
    #[arg(short, long, num_args=1..)]
    /// Tag(s) to categorized a task
    tags: Option<Vec<String>>,
    #[clap(short, long, default_value_t = DateTimeWrapper::default())]
    /// Deadline by which this task should be complete
    deadline: DateTimeWrapper,
}

#[derive(Clone, Debug)]
struct DateTimeWrapper(DateTime<Local>);

impl Default for DateTimeWrapper {
    fn default() -> Self {
        Self(
            Local::now()
                .with_time(
                    NaiveTime::from_hms_opt(17, 0, 0).expect("Hardcoded values")
                )
                .single()
                .expect("Hardcoded values"),
        )
    }
}

impl Display for DateTimeWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d %H:%M"))
    }
}

impl FromStr for DateTimeWrapper {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M";
        const DATE_FORMAT: &str = "%Y-%m-%d";
        let local_offset = Local::now().offset().fix();

        if let Ok(naive_datetime) = NaiveDateTime::parse_from_str(s, DATETIME_FORMAT) {
            match local_offset.from_local_datetime(&naive_datetime).single() {
                Some(datetime) => Ok(DateTimeWrapper(datetime.with_timezone(&Local))),
                None => Err("CannotBeParsedToTimezone".into()),
            }
        } else if let Ok(Some(naive_datetime)) =
            NaiveDate::parse_from_str(s, DATE_FORMAT).map(|date| date.and_hms_opt(17, 0, 0))
        {
            match local_offset.from_local_datetime(&naive_datetime).single() {
                Some(datetime) => Ok(DateTimeWrapper(datetime.with_timezone(&Local))),
                None => Err("CannotBeParsedToDate".into()),
            }
        } else if let Ok(naive_time) = NaiveTime::from_str(s) {
            match Local::now().with_time(naive_time).single() {
                Some(datetime) => Ok(DateTimeWrapper(datetime)),
                None => Err("CannotBeParsedToDate".into()),
            }
        } else {
            Err("Cannot be parsed to Date, Time, or DateTime".into())
        }
    }
}

#[derive(clap::Args, Debug)]
struct CompleteTaskArgs {
    #[arg(value_parser)]
    /// id of task to complete
    id: usize,
}

#[derive(clap::Args, Debug)]
struct DeleteTaskArgs {
    #[arg(value_parser)]
    /// id of task to delete
    id: usize,
}

#[derive(clap::Args, Debug)]
struct ListTasksArgs {
    #[arg(short, long)]
    /// Only show items that have not been completed
    incomplete: bool,
    #[arg(short, long)]
    /// Only show items that are overdue
    overdue: bool,
    #[arg(short, long)]
    /// Only show the next `n` items due
    number: bool,
    #[arg(short, long, num_args=1..)]
    /// Only show items with specific tags
    tags: Option<Vec<String>>,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[command(name = "add", about = "Adds a new task to a project")]
    AddTask(AddTaskArgs),
    #[command(name = "complete", about = "Marks an existing task as finished")]
    CompleteTask(CompleteTaskArgs),
    #[command(name = "delete", about = "Removes a task")]
    DeleteTask(DeleteTaskArgs),
    #[command(name = "list", about = "Shows all tasks")]
    ListTasks(ListTasksArgs),
}

fn main() {
    const DEFAULT_FILE: &str = "./database";
    let default_path = PathBuf::from(DEFAULT_FILE);
    let args = Args::parse();

    let file_path = args.file.as_ref().unwrap_or(&default_path);
    let mut plain_text_tracker =
        task_tracker::plain_text_task_tracker::PlainTextTaskTracker::new(file_path);

    let res = match args.command {
        Commands::AddTask(AddTaskArgs {
            name,
            tags,
            deadline,
        }) => plain_text_tracker.add_task(name, tags, deadline.0.into()),
        Commands::CompleteTask(CompleteTaskArgs { id }) => plain_text_tracker.complete_task(id),
        Commands::DeleteTask(DeleteTaskArgs { id }) => plain_text_tracker.delete_task(id),
        // TODO: Implement using the `ListTasksArgs`
        Commands::ListTasks(ListTasksArgs { .. }) => plain_text_tracker.list_task(),
    };

    match res {
        Ok(()) => (),
        Err(e) => eprintln!("Process failed: {e}"),
    }
}
