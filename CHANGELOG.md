# Changelog

## v0.5.0 (2023-10-02)

* Use log files instead of full directories for tasks
* Only use a global history instead of multiple ones

## v0.4.0 (2023-09-29)

* Switch history format to JSON for better inter-opability
* Implement history file exclusive access to avoid filesystem data races

## v0.3.2 (2022-10-07)

* Fix: don't let dangling socket listening threads after client has exited

## v0.3.1 (2022-09-27)

* NEW: Add a `history` subcommand
* NEW: Add a global history file
* Minor: Only use second precision for start and end time in logs
* Minor: Remove sub-seconds in `list` subcommand
* Minor: Update `scheduled` subcommand display
* Minor: Update inserted line in tasks' logs
* Fix: Correctly detect daemon shutdown in `stop` subcommand
* Fix: Remove unneeded datetime formatting functions
* Fix: inverted duration in `scheduled` subcommand

## v0.3.0 (2022-09-22)

* NEW: Print messages datetime in log files
* NEW: Add `logs [<task_name>]` subcommand
* NEW: Unify STDOUT and STDERR log files

## v0.2.1 (2022-09-21)

* NEW: `daemon-start`: added an `--ignore-started` flag
* Minor: added version and author in command-line help
* Fix: removed italic to prevent misalignment from table

## v0.2.0 (2022-09-21)

* First fully-usable version :tada: