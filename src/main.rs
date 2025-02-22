use crate::database::DataBase;
use std::path::Path;
use std::env;

#[derive(Debug)]
enum Commands {
    AddItem{project: String, description: String},
    CompleteItem(u32),
    DeleteItem(u32),
    // TODO: Add function as predicate for filtering?
    ListItems,
}

#[derive(Clone, Debug)]
enum CommandErrors{
    IdParseError,
    InvalidCommand,
    MissingArgument,
}

impl std::error::Error for CommandErrors {}

impl std::fmt::Display for CommandErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IdParseError => write!(f, "IdParseError - Input could not be converted to a valid u32"),
            Self::InvalidCommand => write!(f, "InvalidCommand - The supplied argument is not a supported command"),
            Self::MissingArgument => write!(f, "MissingArgument - An argument was not found"),
        }
    }
}

impl Commands {
    fn new(cmd: &str, args: &mut std::env::Args) -> Result<Self, CommandErrors> {
        match cmd {
            "add" => Ok(Self::AddItem {
                project: args.next().ok_or(CommandErrors::MissingArgument {})?,
                description: args.next().ok_or(CommandErrors::MissingArgument {})?
            }),
            "complete" => Ok(Self::CompleteItem(args
                .next()
                .ok_or(CommandErrors::MissingArgument {})?
                .parse::<u32>()
                .ok()
                .ok_or(CommandErrors::IdParseError {})?)),
            "delete" => Ok(Self::DeleteItem(args
                .next()
                .ok_or(CommandErrors::MissingArgument {})?
                .parse::<u32>()
                .ok()
                .ok_or(CommandErrors::IdParseError {})?)),
            "list" => Ok(Self::ListItems),
            _ => Err(CommandErrors::InvalidCommand),
        }
    }

    fn do_command(self, database: &mut DataBase) {
        match self {
            Self::AddItem {project, description} => database.add_item(project, description),
            Self::CompleteItem(id) => database.complete_item(id),
            Self::DeleteItem(id) => database.delete_item(id),
            Self::ListItems => database.list_items(),
        };
    }
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
        file_path: PathBuf,
        items: Vec<Item>,
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
                "| {:>2} | {:>10} | {:>20} | {:>5} |",
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

fn main() {
    let mut args = env::args();
    let cmd = Commands::new(&args.nth(1).expect(
        "Need a command - one of: add, complete, delete, or list"
    ), &mut args);
    dbg!(&cmd);
    let file_name = Path::new("./database");
    let mut database = DataBase::read(file_name).unwrap();
    dbg!(&database);
    cmd.unwrap().do_command(&mut database);
    dbg!(&database);
}



// #[cfg(test)]
// mod tests {
//     use crate::Item;
//
//     #[test]
//     fn it_works() {}
// }
