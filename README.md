# on_job
## Description
A simple command-line interface to manage your tasks.
You can add, complete, delete, and list your tasks. Tasks are stored in a local database file.

## Usage
```bash
on_job [OPTIONS] <COMMAND>
```
### Options
- `-h --help`
Prints help information

- `-V --version`
Prints version information

- `-f --file <FILE>`
Specify the database file path
- Must be specified if the environment variable `ON_JOB_FILE` is not set

### Commands
#### add
Creates a new, incomplete task
```bash
on_job add --name <NAME> --tags <TAG> [<TAG> ...] --deadline <DEADLINE>
```
##### Options
- `-n --name <NAME>`
The name/description of the task (Required)

- `-t --tags <TAG> [<TAG> ...]`
Tags categorizing the task (Optional)

- `-d --deadline <DEADLINE>`
Deadline by which this task should be complete [default: "2025-03-07 17:00"]
    - Can be specified in three ways
        - Date: "YYYY-DD-MM"
        - Time: "HH:MM"
        - Date and time: "YYYY-DD-MM HH:MM"

##### Examples
```bash
on_job add --name "Shuffle papers around" --tags "Busy work"
```
Adds one task with one tag due at the end of the day (current day, 17:00)

```bash
on_job add -n "Pickup Coffee" -d "13:00"
```
Adds one task without a tag due at 13:00 (1pm)

```bash
on_job add -n "Meet with client" -t "External" "High stakes" --deadline "2025-03-10"
```
Adds one task with multiple tags due at the end of the day on March 10

```bash
on_job add -n "Avoid client interaction" --deadline "2099-12-31 23:59"
```
Adds one task without any tags due at the last minute of 2099

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
```bash
on_job list
```
Shows incomplete tasks (by default)

##### Options
- `-a --all`
Shows completed tasks in addition to incomplete tasks
- If passed, removes the id column as well. This is to avoid one
task having differing ids in different contexts

- `-o --overdue`
Filters list to show tasks for which the deadline has passed

- `-t --tags <TAG> [<TAG> ...]`
Filters list to only show tasks with all of the specified tags

##### Examples

```bash
on_job list -a
```
Shows all tasks

```bash
on_job list -o -t "Project 1"
```
Shows all overdue tasks for Project 1

```bash
on_job list -a -t "A" "B"
```
Shows all tasks with tags A and B

## Roadmap
- [x] Add time (deadline) information to tasks
    - [x] Specify as a date and time
        - "YYYY-DD-MM: HH:MM"
    - [x] Specify as a date only
        - "YYYY-DD-MM"
    - [x] Specify as a time only
        - "HH:MM"
    - [ ] Add additional pre-defined markers
        - End of day (default)
        - End of week
        - End of hour
        - Tomorrow morning
        - etc.
- [x] Add ability to filter which tasks are shown
    - [x] Only incomplete tasks
    - [x] Only tasks overdue/due today
- [x] Cleanup
    - [x] Change default file to use environment variable
