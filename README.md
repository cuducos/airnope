# AirNope

A simple, silent bot that keeps [Telegram](https://telegram.org/) groups free from crypto airdrop spams.

## What it does

When the user posting the message is **not** one of the group admins or the group owner:

 * deletes the message if it contains “airdrop” (including many character variants)
 * removes the user who posted it from the group

## What it does not do

* does **not** post any message in the group (avoids pollution of the group)
* does **not** keep any history of messages or users

## Privacy concerns

AirNope is designed to detect spam messages, and in some cases, it might log them for debugging purposes. While logging these messages, there is a possibility that personally identifiable information (PII) might be inadvertently captured. We understand the importance of privacy and are committed to ensuring that any PII collected is not processed or persisted. Logs are temporary and are deleted periodically, either during each release cycle or when the bot is restarted.

We are also considering the creation of a database of spam messages to further enhance our spam detection capabilities. However, due to our concern for user privacy and the potential risk of PII exposure, this initiative is not currently part of our roadmap. We will continue to prioritize privacy and will take all necessary measures to protect user information should this project be considered in the future.

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
