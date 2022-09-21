# Crony

Crony is a replacement tool for the famous `cron` as well as `crontab`. It provides a considerable list of advantages compared to the original:

* [Fits in a single, ~1 MB executable](https://github.com/ClementNerma/Crony/releases/latest)
* Clear repetition syntax, immediatly readable and not error-prone
* Ability to name tasks
* Ability to list all tasks, see their last execution date and exit status
* Ability to register and unregister tasks without using a text edit
* All tasks' STDOUT and STDERR are logged to files by default
* Complete execution history with start and end times as well as exit code
* Direct execution of a task to test if it's working
* Portable tasks

You can find the list of changes in the [releases page](https://github.com/ClementNerma/Crony/releases/latest) or directly in the [Changelog](CHANGELOG.md).

## Using the daemon

To start Crony:

```shell
crony daemon-start
```

To check the daemon's status at anytime:

```shell
crony daemon-status
```

To stop the daemon:

```shell
crony daemon-stop
```

Note that the daemon will wait until all tasks have properly exited before stopping itself. This ensures that no task is interrupted in the middle of its process.

By default, all of the daemon's and task's data (history, log files, ...) is stored in `~/.config/crony`. You can override this setting by providing a `--data-dir <path>` argument.

This directory is portable, meaning that copying it on another machine will automatically restore all your tasks, history and log files.

## Managing tasks

Tasks can be registered through the `register` subcommand:

```shell
crony register <task_name> --run <command with arguments> --at <repetition pattern> [--using <shell command>]
```

Here is how we register a task displaying `Hello world` every minutes:

```shell
crony register hello-world --run "echo 'Hello world'" --at "m=*"
```

To provide use a custom shell (default is `/bin/sh -c`):

```shell
crony register hello-world --run "echo 'Hello world'" --at "m=*" --using "/bin/zsh -c"
```

If we want to remove the task:

```shell
crony unregister hello-world
```

Note that any registering / unregistering action will contact the daemon to ensure it reloads correctly.

## Repetition patterns

A repetition pattern indicates when a task should be run.

You can specify any of the following letters:

* `M` for months
* `D` for days
* `h` for hours
* `m` for minutes
* `s` for seconds

Followed by a `=` and a value, which can either be!

* A fixed value (e.g. running every day at 3 AM will translate to `h=3`)
* A list of values, separated by a comma (e.g. running at 3 AM **and** 6 AM will translate to `h=3,6`)
* A wildcard `*` to indicate it should run at every occurrence

You can specify multiple letters, separated by a space. They MUST be in the presented order (first `M`, then `D`, then `h`, etc.).

Here is an example to run a task every day of february, at 1 PM:

```
M=2 D=* h=13
```

**NOTE:** Tasks cannot overlap, which means that if a task is scheduled to run every minute but it takes 3 minutes to complete, it will not be the run on the second and third minute as it has not completed yet. After the task completes, it is re-scheduled as usual.

## Test a command

To run a command directly:

```shell
crony run <task name>
```

This will execute the task immediatly and show the output in the terminal. If you want to write the output to the task's log files instead, use `--log-files`.
