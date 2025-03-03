use clap::Parser;
use std::path::PathBuf;
use task_tracker::TaskTracker;
mod task_tracker;

#[derive(Debug, Parser)]
#[command(
    author="Dominic Culotta",
    version="0.2.0",
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
struct AddItemArgs {
    #[arg(short, long)]
    /// Name of project
    name: String,
    #[arg(short, long, num_args=1..)]
    /// Tag(s) to categorized a task
    tags: Option<Vec<String>>,
}

#[derive(clap::Args, Debug)]
struct CompleteItemArgs {
    #[arg(value_parser)]
    /// id of task to complete
    id: usize,
}

#[derive(clap::Args, Debug)]
struct DeleteItemArgs {
    #[arg(value_parser)]
    /// id of task to delete
    id: usize,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[command(name = "add", about = "Adds a new task to a project")]
    AddItem(AddItemArgs),
    #[command(name = "complete", about = "Marks an existing task as finished")]
    CompleteItem(CompleteItemArgs),
    #[command(name = "delete", about = "Removes a task")]
    DeleteItem(DeleteItemArgs),
    #[command(name = "list", about = "Shows all tasks")]
    ListItems,
}

fn main() {
    const DEFAULT_FILE: &str = "./database";
    let default_path = PathBuf::from(DEFAULT_FILE);
    let args = Args::parse();
    let file_path = args.file.as_ref().unwrap_or(&default_path);
    // TODO: Determine how to define which task tracker to use
    let mut plain_text_tracker =
        task_tracker::plain_text_task_tracker::PlainTextTaskTracker::new(file_path);

    let res = match args.command {
        Commands::AddItem(AddItemArgs { name, tags }) => plain_text_tracker.add_item(name, tags),
        Commands::CompleteItem(CompleteItemArgs { id }) => plain_text_tracker.complete_item(id),
        Commands::DeleteItem(DeleteItemArgs { id }) => plain_text_tracker.delete_item(id),
        _ => todo!(),
    };

    // TODO: Change error handling
    match res {
        Ok(_) => println!("Added item successfully"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
