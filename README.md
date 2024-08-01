# AirNope

A simple, silent bot that keeps [Telegram](https://telegram.org/) groups free from crypto airdrop spams.

## What it does

When the user posting the message is **not** one of the group admins or the group owner:

 * deletes the message if it containing “airdrop” (including many character variants)
 * removes the user who posted it from the group

## What it does not do

* does **not** post any message in the group (avoids pollution of the group)
* does **not** keep any history of messages or users

## How to use it

1. Add [`@airnope_bot`](https://telegram.me/airnope_bot) to your group
2. Make [`@airnope_bot`](https://telegram.me/airnope_bot) an admin able to delete messages and remove users

## Developing and contributing

### Requirements

AirNope requires an environment variable called `TELOXIDE_TOKEN` with the [Telegram API token](https://core.telegram.org/bots/#how-do-i-create-a-bot).

### Running it

To avoid attackers trying to bring the server down, this bot does not use a webhook strategy. To start the daemon, run:

```console
$ cargo run
```

Or, if you don't have [`rustup`](https://www.rust-lang.org/tools/install), you can use the container image:

```console
$ docker build -t airnope .
$ docker run -e TELOXIDE_TOKEN=<YOUR TELEGRAM API TOKEN> -d airnope
```

### Before opening a PR

Make sure these checks pass:

```console
$ cargo test
$ cargo clippy
```
