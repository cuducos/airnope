## Developing and contributing

## Requirements

* [`rustup`](https://www.rust-lang.org/tools/install)
* an environment variable called `TELOXIDE_TOKEN` with the [Telegram API token](https://core.telegram.org/bots/#how-do-i-create-a-bot).

## Running the bot

To avoid attackers trying to bring the server down, this bot does not use a webhook strategy. To start the daemon, run:

```console
$ cargo run
```

## Running the REPL

For developing and manual testing, there is a [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop). No Telegram token is required. Set the environment variable `RUST_LOG` to `airnope=debug` to see extra information.

```console
$ cargo run -- --repl
```

## Before opening a PR

Make sure these checks pass:

```console
$ cargo test
$ cargo clippy
```
