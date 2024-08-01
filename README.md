# AirNope

A simple and silent bot that keeps [Telegram's](https://telegram.org/) groups free from crypto airdrop spams.

## What it does

 * deletes messages containing “airdrop” (including many characters variants)
 * removes the user who posted it from the group

## What it does not do

* does **not** post any message in the group (avoids pollution of the group)
* does **not** keep any history of messages or users

## How to use it

1. Add [`@airnope_bot`](https://telegram.me/airnope_bot) to your group
2. Make [`@airnope_bot`](https://telegram.me/airnope_bot) an admin able to delete messages and remove users

## Developing and contributing

Requires an environment variable called `TELOXIDE_TOKEN` with the Telegram API token.

```console
$ cargo test
$ cargo clippy
```
