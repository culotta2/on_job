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
    #[arg(value_parser)]
    /// id of task to complete
    id: u32,
}

#[derive(clap::Args, Debug)]
struct DeleteItemArgs {
    #[arg(value_parser)]
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

    #[derive(Debug, Default, PartialEq, Eq)]
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
                id: self.get_max_id().map(|v| v + 1).unwrap_or(0),
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

        pub fn get_max_id(&self) -> Option<u32> {
            self.items
                .iter()
                .map(|x| x.id)
                .max()
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

    impl FromStr for DataBase {
        type Err = InvalidItem;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(
                DataBase {
                    file_path: PathBuf::new(),
                    items: s
                        .lines()
                        .map(|line| line.parse())
                        .collect::<Result<Vec<Item>, InvalidItem>>()?
                }
            )
        }
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct Item {
        id: u32,
        project: String,
        description: String,
        complete: bool,
        // time_initialized: Time,
    }

    impl Item {
        pub fn new(id: u32, project: String, description: String, complete: bool) -> Self {
            Item { id, project, description, complete }
        }
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

    #[derive(Debug)]
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
                .trim()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Item;
    #[test]
    fn test_database_add_to_empty_database() {
        let mut result_db = DataBase::default();
        let expected_db = DataBase {
            file_path: "".into(),
            items: vec![Item::new(0, "Project Time".into(), "Top tier description".into(), false)],
        };
        result_db.add_item("Project Time".into(), "Top tier description".into());
        assert_eq!(expected_db, result_db);
    }

    #[test]
    fn test_database_add_to_database() {
        let mut result_db = DataBase {
            file_path: "".into(),
            items: vec![
                Item::new(0, "Project 1".into(), "First task".into(), true),
                Item::new(1, "Project 1".into(), "Second task".into(), false),
                Item::new(2, "Project 2".into(), "New project task".into(), true),
            ],
        };
        let expected_db = DataBase {
            file_path: "".into(),
            items: vec![
                Item::new(0, "Project 1".into(), "First task".into(), true),
                Item::new(1, "Project 1".into(), "Second task".into(), false),
                Item::new(2, "Project 2".into(), "New project task".into(), true),
                Item::new(3, "Project 1".into(), "Same old same old".into(), false),
            ],
        };
        result_db.add_item("Project 1".into(), "Same old same old".into());
        assert_eq!(expected_db, result_db);
    }

    #[test]
    fn test_database_complete_item_empty_database() {
        let expected_db = DataBase::default();
        let mut result_db = DataBase::default();
        result_db.complete_item(100);
        assert_eq!(expected_db, result_db);
    }

    #[test]
    fn test_database_complete_item() {
        let expected_db = DataBase {
            file_path: "".into(),
            items: vec![
                Item::new(0, "Project 1".into(), "First task".into(), true),
                Item::new(1, "Project 1".into(), "Second task".into(), false),
                Item::new(2, "Project 2".into(), "New project task".into(), true),
                Item::new(3, "Project 1".into(), "Same old same old".into(), false),
            ],
        };
        let mut result_db = DataBase {
            file_path: "".into(),
            items: vec![
                Item::new(0, "Project 1".into(), "First task".into(), true),
                Item::new(1, "Project 1".into(), "Second task".into(), false),
                Item::new(2, "Project 2".into(), "New project task".into(), false),
                Item::new(3, "Project 1".into(), "Same old same old".into(), false),
            ],
        };
        result_db.complete_item(2);
        assert_eq!(expected_db, result_db);
    }

    #[test]
    fn test_database_delete_item_empty_database() {
        let expected_db = DataBase::default();
        let mut result_db = DataBase::default();
        result_db.delete_item(100);
        assert_eq!(expected_db, result_db);
    }

    #[test]
    fn test_database_delete_item() {
        let expected_db = DataBase {
            file_path: "".into(),
            items: vec![
                Item::new(0, "Project 1".into(), "First task".into(), true),
                Item::new(1, "Project 1".into(), "Second task".into(), false),
                Item::new(3, "Project 1".into(), "Same old same old".into(), false),
            ],
        };
        let mut result_db = DataBase {
            file_path: "".into(),
            items: vec![
                Item::new(0, "Project 1".into(), "First task".into(), true),
                Item::new(1, "Project 1".into(), "Second task".into(), false),
                Item::new(2, "Project 2".into(), "New project task".into(), true),
                Item::new(3, "Project 1".into(), "Same old same old".into(), false),
            ],
        };
        result_db.delete_item(2);
        assert_eq!(expected_db, result_db);
    }

    #[test]
    fn test_create_database_from_string() {
        let test_str = "| 0 | Project 1 | Doing my job | true |\n | 1 | Project 1 | Job time | true |\n | 2 | Project 2 | What time is it? | true |\n | 3 | Project 3 | Time for job | false |";
        let expected_db = DataBase {
            file_path: std::path::PathBuf::new(),
            items: vec![
                Item::new(0, "Project 1".into(), "Doing my job".into(), true),
                Item::new(1, "Project 1".into(), "Job time".into(), true),
                Item::new(2, "Project 2".into(), "What time is it?".into(), true),
                Item::new(3, "Project 3".into(), "Time for job".into(), false),
            ],
        };
        let result_db: DataBase = test_str.parse().expect("Did not parse successfully");
        assert_eq!(expected_db, result_db);
    }
}
