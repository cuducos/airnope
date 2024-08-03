# AirNope

A simple, silent bot that keeps [Telegram](https://telegram.org/) groups free from crypto airdrop spams.

## What is AirNope?

### What it does

When the user posting the message is **not** one of the group admins or the group owner:

 * deletes the message that are probably airdrop spam
 * removes the user who posted it from the group

#### How it does

The detection is done in two steps:

1. the first filter detects if the message contains the word “airdrop” (including many character variants)
2. the second step is a [zero-shot classification](https://huggingface.co/tasks/zero-shot-classification) to identify the probability of the message being a spam related to crypto airdrop

Wanna see it in action? Check the how to run [REPL in our contributing guide](CONTRIBUTING.md#running-the-repl).

### What it does not do

* does **not** post any message in the group (avoids pollution of the group)
* does **not** keep any history of messages or users

### Privacy concerns

AirNope is designed to detect spam messages, and in some cases, it might log them for debugging purposes. While logging these messages, there is a possibility that personally identifiable information (PII) might be inadvertently captured. We understand the importance of privacy and are committed to ensuring that any PII collected is not processed or persisted. Logs are temporary and are deleted periodically, either during each release cycle or when the bot is restarted.

We are also considering the creation of a database of spam messages to further enhance our spam detection capabilities. However, due to our concern for user privacy and the potential risk of PII exposure, this initiative is not currently part of our roadmap. We will continue to prioritize privacy and will take all necessary measures to protect user information should this project be considered in the future.

## How to use AirNope?

1. Add [`@airnope_bot`](https://telegram.me/airnope_bot) to your group
2. Make [`@airnope_bot`](https://telegram.me/airnope_bot) an admin able to delete messages and remove users
