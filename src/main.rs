use crate::database::DataBase;
use std::{fs::OpenOptions, path::Path};
use std::io::prelude::*;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author="Dominic Culotta",
    version="0.1.0",
    about="A todo CLI application",
    long_about = None
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Args, Debug)]
struct AddItemArgs {
    #[arg(short, long)]
    /// Name of project
    project: String,
    #[arg(short, long)]
    /// Description of task
    description: String,
}

#[derive(clap::Args, Debug)]
struct CompleteItemArgs {
    #[arg(short, long)]
    /// id of task to complete
    id: u32,
}

#[derive(clap::Args, Debug)]
struct DeleteItemArgs {
    #[arg(short, long)]
    /// id of task to delete
    id: u32,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[command(name="add", about="Adds a new task to a project")]
    AddItem(AddItemArgs),
    #[command(name="complete", about="Marks an existing task as finished")]
    CompleteItem(CompleteItemArgs),
    #[command(name="delete", about="Removes a task")]
    DeleteItem(DeleteItemArgs),
    #[command(name="list", about="Shows all tasks")]
    ListItems,
}


mod database {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        path::{Path, PathBuf},
        str::FromStr,
    };

    #[derive(Debug)]
    pub struct DataBase {
        pub file_path: PathBuf,
        pub items: Vec<Item>,
    }

    impl DataBase {
        // TODO: Make database work with standard input
        // - generalize the input that database can take
        // will make for a more flexible CLI tool
        pub fn read(file_path: &Path) -> Option<Self> {
            let file = match file_path.exists() {
                true => File::open(file_path),
                false => File::create_new(file_path),
            }
            .ok()?;
            let reader = BufReader::new(file);
            let items = reader
                .lines()
                .map_while(Result::ok)
                .map(|x| x.parse::<Item>())
                .collect::<Result<Vec<Item>,InvalidItem>>()
                .ok()?
            ;

            Some(DataBase {
                file_path: file_path.into(),
                items,
            })
        }

        pub fn add_item(&mut self, project: String, description: String) {
            self.items.push(Item {
                id: self.get_max_id() + 1,
                project,
                description,
                complete: false,
            });
        }

        pub fn complete_item(&mut self, id: u32) {
            self.items
                .iter_mut()
                .filter(|item| item.id == id)
                .for_each(|item| item.complete = true)
            ;
        }

        pub fn delete_item(&mut self, id: u32) {
            self.items
                .retain_mut(|x| x.id != id)
        }

        pub fn get_max_id(&self) -> u32 {
            self.items
                .iter()
                .map(|x| x.id)
                .max()
                .unwrap_or(0)
        }

        pub fn list_items(&self) {
            // TODO: Format header to be pretty
            println!("| --- | ---------- | -------------------- | -------- |");
            println!("| ID  | Project    | Description          | Complete |");
            println!("| --- | ---------- | -------------------- | -------- |");
            self.items
                .iter()
                .for_each(|item| println!("{item}"))
            ;
        }

    }

    #[derive(Debug)]
    pub struct Item {
        id: u32,
        project: String,
        description: String,
        complete: bool,
        // time_initialized: Time,
    }

    impl std::fmt::Display for Item {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f,
                // TODO: Make padding dynamic
                "| {:<3} | {:<10} | {:<20} | {:<8} |",
                self.id,
                self.project,
                self.description,
                self.complete,
            )
        }
    }

    pub struct InvalidItem {}

    impl From<Item> for String {
        fn from(value: Item) -> Self {
            format!(
                "| {} | {} | {} | {} |",
                value.id,
                value.project,
                value.description,
                value.complete,
            )
        }
    }

    impl FromStr for Item {
        type Err = InvalidItem;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let vals: Vec<_> = s
                .trim_matches('|')
                .trim()
                .split("|")
                .map(|x| x.trim())
                .collect();

            if let [id, project, description, complete] = vals[..] {
                let id = match id.parse::<u32>() {
                    Ok(v) => v,
                    Err(_) => return Err(InvalidItem {}),
                };

                let complete = match complete.parse::<bool>() {
                    Ok(v) => v,
                    Err(_) => return Err(InvalidItem {}),
                };

                Ok(Item {
                    id,
                    project: project.into(),
                    description: description.into(),
                    complete,
                })
            } else {
                Err(InvalidItem {})
            }
        }
    }
}

fn write_database_to_file(database: DataBase) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&database.file_path)
    ?;

    for item in database.items {
        writeln!(file, "{}", String::from(item))?;
    };

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let file_name = Path::new("./database");
    let mut database = DataBase::read(file_name).unwrap();
    let args = Args::parse();

    match args.command {
        Commands::AddItem(add_item_args) => database.add_item(add_item_args.project, add_item_args.description),
        Commands::CompleteItem(complete_item_args) => database.complete_item(complete_item_args.id),
        Commands::DeleteItem(delete_item_args) => database.delete_item(delete_item_args.id),
        Commands::ListItems => database.list_items(),
    }

    write_database_to_file(database)?;

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use crate::Item;
//
//     #[test]
//     fn it_works() {}
// };
