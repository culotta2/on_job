# on_job
## Description
A simple command-line interface to manage your tasks.
You can add, complete, delete, and list your tasks. Tasks are stored in a local database file.

## Usage
```bash
on_job [OPTIONS] <COMMAND>
```
### Options
- `-f --file <FILE>`
Specify the database file path. By default, a file called `database` is used in the current directory (*TO BE CHANGED*)

- `-h --help`
Prints help information

- `-V --version`
Prints version information

### Commands
#### add
Creates a new, incomplete task
```bash
on_job add --project <PROJECT> --description <DESCRIPTION>
```
##### Options
- `-p, --project <PROJECT>`
The name of the project (category) to which to assign this task (Required)

- `-d, --description <DESCRIPTION>`
A description of the task (Required)

##### Examples
```bash
on_job add --project "My Client" --description "Do my job"
```

```bash
on_job add "Other client" -d "Do my other job"
```

#### complete
Marks a preexisting task as finished
```bash
on_job complete <ID>
```
##### Options
- `<ID>`
The unique identifier for the task to be completed (Required)
    - Must be a positive integer

##### Examples
```bash
on_job complete 42
```
The task with id 42 is marked as complete
- If task 42 is already marked complete, it replaces the `true` with the same value
- If task 42 does not exist, does nothing


#### delete
Removes a preexisting task from the task list
```bash
on_job delete <ID>
```
##### Options
- `<ID>`
The unique identifier for the task to be completed (Required)
    - Must be a positive integer

##### Examples
```bash
on_job delete 42
```
The task with id 42 is removed from the list
- If task 42 does not exist, does nothing
- Does not update subsequent tasks' id values

#### list
Shows all tasks
```bash
on_job list
```


## Roadmap
- [ ] Change project/description format to name/tags format
- [ ] Change default file to use environment variable
- [ ] Additional storage options
    - [x] Local text file (delimited by ` | `)
    - [ ] sqlite database
- [ ] Add time information to tasks
    - [ ] Specify this in the following ways
        - [ ] absolute time (14:00)
        - [ ] relative time (1hr)
        - [ ] defined markers
            - `EOD`: End of day (17:00)
            - `MOR`: Next morning (8:00)
            - `EOH`: End of hour (at 9:34 -> 10:00)
            - etc.
- [ ] Add ability to filter which tasks are shown
    - [ ] Only incomplete tasks
    - [ ] Only tasks overdue/due today
