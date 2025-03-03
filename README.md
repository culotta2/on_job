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
on_job add --name <NAME> --tags <TAG> [<TAG> ...]
```
##### Options
- `-n --name <NAME>`
The name/description of the task (Required)

- `-t --tags <TAG> [<TAG> ...]`
Tags categorizing the task (Optional)

##### Examples
```bash
on_job add --name "Shuffle papers around" --tags "Busy work"
```
Adds one task with one tag

```bash
on_job add -n "Pickup Coffee"
```
Adds one task without a tag

```bash
on_job add -n "Meet with client" -t "External" "High stakes"
```
Adds one task with multiple tags

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
- [ ] Cleanup
    - [ ] Change default file to use environment variable
- [ ] Add time (deadline) information to tasks
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
### Potential features
- [ ] Additional storage options
    - [ ] sqlite database
