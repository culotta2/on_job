use database::DataBase;
use std::path::Path;

mod database {
    use std::{
        fs::File,
        io::{BufRead, BufReader, BufWriter, Write},
        path::{Path, PathBuf},
        str::FromStr,
    };

    #[derive(Debug)]
    pub struct DataBase {
        file_name: PathBuf,
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
                file_name: file_path.into(),
                items,
            })
        }
    }

    #[derive(Debug)]
    // TODO: Consider removing the id
    pub struct Item {
        id: u32,
        project: String,
        description: String,
        complete: bool,
        // time_initialized: Time,
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
    let file_name = Path::new("./database");
    // let database = DataBase::read(file_name).expect("File not found");
    let database = DataBase::read(file_name);
    dbg!(database);
}

// #[cfg(test)]
// mod tests {
//     use crate::Item;
//
//     #[test]
//     fn it_works() {}
// }
