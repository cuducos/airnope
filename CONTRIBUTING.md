## Developing and contributing

## Requirements

* [`rustup`](https://www.rust-lang.org/tools/install)
* an environment variable called `TELOXIDE_TOKEN` with the [Telegram API token](https://core.telegram.org/bots/#how-do-i-create-a-bot).

## Running the bot

The bot can be ran as a webhook or using the [long pooling strategy](https://core.telegram.org/bots/api#getupdates):

```console
$ cargo run -- bot --mode webhook
$ cargo run -- bot --mode long-pooling
```

If there is no flag `--mode`:

* It starts as a webhook if both `PORT` and `HOST` environment variables are set
* It starts using the long pooling strategy otherwise

When running as webhook, AirNope register its URL (and secret token) with Telegram servers. During a graceful shutdown, AirNope [removes the webhook](https://core.telegram.org/bots/api#deletewebhook) from Telegram servers, so you can go back to long pooling if needed.

## Running the REPL

For developing and manual testing, there is a [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop). No Telegram token is required. Set the environment variable `RUST_LOG` to `airnope=debug` to see extra information.

```console
$ cargo run -- repl
```

## Playing with the zero-shot classifier

This classifier is based on a label, which is a constant in AirNope. You can benchmark alternative labels with the option `--bench` and passing alternative labels, for example:

```console
$ cargo run -- bench "airdop spam" "generic spam offering crypto airdrop"

==> Reference: crypto airdrop spam message (threshold: 0.55)
    ✔ not_spam1.txt 0.445 (-0.105)
    ✔ spam1.txt     0.825 (+0.275)
    ✔ spam2.txt     0.626 (+0.076)
    ✔ spam3.txt     0.724 (+0.174)
    ✔ spam4.txt     0.567 (+0.017)
    ✔ spam5.txt     0.635 (+0.085)
    ✔ spam6.txt     0.592 (+0.042)
    ✔ spam7.txt     0.573 (+0.023)

==> Alternative 1: airdop spam
    ✘ not_spam1.txt 0.613 (+0.063)
    ✔ spam1.txt     0.807 (+0.257)
    ✔ spam2.txt     0.712 (+0.162)
    ✔ spam3.txt     0.800 (+0.250)
    ✔ spam4.txt     0.652 (+0.102)
    ✔ spam5.txt     0.691 (+0.141)
    ✔ spam6.txt     0.644 (+0.094)
    ✔ spam7.txt     0.612 (+0.062)

==> Alternative 2: generic spam offering crypto airdrop
    ✔ not_spam1.txt 0.468 (-0.082)
    ✔ spam1.txt     0.793 (+0.243)
    ✔ spam2.txt     0.662 (+0.112)
    ✔ spam3.txt     0.750 (+0.200)
    ✔ spam4.txt     0.571 (+0.021)
    ✔ spam5.txt     0.553 (+0.003)
    ✘ spam6.txt     0.507 (-0.043)
    ✔ spam7.txt     0.581 (+0.031)
```

To test combined labels, separate them with commas inside the quotes, for example:

```console
$ cargo run --bin airnope-bench "airdop spam" "generic spam, crypto airdrop offer"
```

## Before opening a PR

Make sure these checks pass:

```console
$ cargo test
$ cargo clippy
```
