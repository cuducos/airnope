## Developing and contributing

## Requirements

AirNope requires an environment variable called `TELOXIDE_TOKEN` with the [Telegram API token](https://core.telegram.org/bots/#how-do-i-create-a-bot).

## Running it

To avoid attackers trying to bring the server down, this bot does not use a webhook strategy. To start the daemon, run:

```console
$ cargo run
```

Or, if you don't have [`rustup`](https://www.rust-lang.org/tools/install), you can use the container image:

```console
$ docker build -t airnope .
$ docker run -e TELOXIDE_TOKEN=<YOUR TELEGRAM API TOKEN> -d airnope
```

## Before opening a PR

Make sure these checks pass:

```console
$ cargo test
$ cargo clippy
```
