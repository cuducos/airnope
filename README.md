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

### What it does not* do

* does **not** post any message in the group (avoids pollution of the group)
* does **not** keep any history of messages or users

## How to use AirNope?

1. Add [`@airnope_bot`](https://telegram.me/airnope_bot) to your group
2. Make [`@airnope_bot`](https://telegram.me/airnope_bot) an admin able to delete messages and remove users

## FAQ

<details>

<summary>Is there a privacy policy?</summary>

AirNope is designed to detect spam messages, and in some cases, it might log them for debugging purposes. While logging these messages, there is a possibility that personally identifiable information (PII) might be inadvertently captured. We understand the importance of privacy and are committed to ensuring that any PII collected is not processed or persisted. Logs are temporary and are deleted periodically, either during each release cycle or when the bot is restarted.

We are also considering the creation of a database of spam messages to further enhance our spam detection capabilities. However, due to our concern about user privacy and the potential risk of PII exposure, this initiative is not currently part of our roadmap. We will continue to prioritize privacy and will take all necessary measures to protect user information should this initiative be considered in the future.

</details>

<details>

<summary>Can I test it to see what messages AirNope would consider spam?</summary>

Sure! The easiet way to use the [playground](https://airnope-playground.onrender.com).

The second easiest way is to create a group and [add AirNope](#how-to-use-airnope). Since you would then be the group owner, you will need a second account (friends!) to join the group to see the bot in action.

Alternatively, you can use [Docker](https://docs.docker.com/get-started/) and your terminal to test messages locally:

First, download the Docker image:

```console
$ docker pull ghcr.io/cuducos/airnope:main
```

Then start the [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop):

```console
$ docker run -it -e RUST_LOG="airnope=debug" ghcr.io/cuducos/airnope:main airnope-repl
```

It is a long command, but let's break it down:

* `docker run` runs a Docker image
* `-it` sets it to be interactive (menaing, you cna type stuff in the execution from your terminal)
* `-e RUST_LOG="airnope-debug"` is **optional**, it makes information about each classifier visible in the output, so you can know which step flagged the message as spam)
* `cuducos/airnope:main` is the container image we are using
* finalle, `airnope-repl` is the command we are running inside that container image

It is interactive, so you can type anything. Here is how it looks like with three messages to illustrate it:

```
Type `exit` to quit.
> Hello, folks!
Not spam
> Can we talk about airdrop in this group?
DEBUG airnope::re  > Message detected as spam by RegularExpression: "Can we talk about airdrop in this group?"
Not spam
> The Q Community Аirdrop takes us through а journey in time. During its three seаsons, Q will rewаrd the people who аre relentlessly building towаrds а better future. In totаl, Q Internаtionаl Foundаtion is distributing 10% of the initiаl totаl QGOV token supply
DEBUG airnope::re  > Message detected as spam by RegularExpression: "he Q Community Аirdrop takes us through а journey in time. During its three seаsons, Q will rewаrd the people who аre relentlessly building towаrds а better future. In totаl, Q Internаtionаl Foundаtion is distributing 10% of the initiаl totаl QGOV token supply"
DEBUG airnope::zsc > Message detected as spam by ZeroShotClassification (score = 0.7079526): "The Q Community Аirdrop takes us through а journey in time. During its three seаsons, Q will rewаrd the people who аre relentlessly building towаrds а better future. In totаl, Q Internаtionаl Foundаtion is distributing 10% of the initiаl totаl QGOV token supply"
Spam
>
```

</details>

<details>

<summary>Can I run my own instance of AirNope?</summary>

Absolutely and it is really simple:

1. Create a Telegram bot saving your Telegram API token
2. Download the Docker image with `docker pull ghcr.io/cuducos/airnope:main`
3. Start the bot with `docker run -e TELOXIDE_TOKEN=<TOKEN> ghcr.io/cuducos/airnope:main`

</details>
