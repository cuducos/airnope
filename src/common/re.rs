use crate::truncated;
use anyhow::Result;
use regex::{Regex, RegexBuilder};

const A: &str = "[аa🅰🅰️🇦🇦]";
const D: &str = "[dԁ🇩]";
const E: &str = "[eE3€ℯ🇪]";
const I: &str = "[іiI1lℹ️🇮]";
const K: &str = "[kK🇰]";
const L: &str = "[lL1|ℓ🇱]";
const N: &str = "[nNℕ🇳]";
const O: &str = "[оo0🅾️🇴]";
const P: &str = "[рpρϱ🅿️🇵]";
const R: &str = "[рr🇷]";
const T: &str = "[tTТ7†🇹]";
const W: &str = "[wW🇼]";

#[derive(Clone)]
pub struct RegularExpression {
    airdrop: Regex,
    wallet: Regex,
    token: Regex,
    cleanup: Regex,
}

fn to_regex<I>(chars: I) -> Result<Regex>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    Ok(RegexBuilder::new(
        chars
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect::<Vec<_>>()
            .join(r"\s?")
            .as_str(),
    )
    .case_insensitive(true)
    .build()?)
}

impl RegularExpression {
    pub async fn new() -> Result<Self> {
        let airdrop = to_regex([A, I, R, D, R, O, P])?;
        let wallet = to_regex([W, A, L, L, E, T])?;
        let token = to_regex([T, O, K, E, N])?;
        let cleanup = Regex::new(r"\s")?;
        Ok(Self {
            airdrop,
            wallet,
            token,
            cleanup,
        })
    }

    pub async fn is_spam(&self, txt: &str) -> Result<bool> {
        let cleaned = self.cleanup.replace_all(txt, " ").to_string();
        let result = self.airdrop.is_match(cleaned.as_str())
            || (self.wallet.is_match(cleaned.as_str()) && self.token.is_match(cleaned.as_str()));
        if result {
            log::info!("Message detected as spam by RegularExpression");
            log::debug!("{}", truncated(txt));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_spam() {
        let test_cases = vec![
            ("airdrop", true),
            ("аirdrop", true), // Cyrillic а
            ("аirdrор", true), // Cyrillic а and Cyrillic о
            ("аirdrоp", true), // Cyrillic а and Cyrillic о
            ("аirdrор", true), // Cyrillic а, Cyrillic о, and Cyrillic р
            ("airdrор", true), // Cyrillic о
            ("airdrоp", true), // Cyrillic о
            ("airdrор", true), // Cyrillic о and Cyrillic р
            ("аirdrоp", true), // Cyrillic а, and Cyrillic о
            ("аirdrор", true), // Cyrillic а, Cyrillic о, and Cyrillic р
            ("аirdrop", true), // Cyrillic а
            ("аirdrор", true), // Cyrillic а and Cyrillic р
            ("аirdrop", true), // Cyrillic а
            ("airԁrop", true), // Cyrillic ԁ
            ("aіrԁrop", true), // Cyrillic і and Cyrillic ԁ
            ("airԁroр", true), // Cyrillic р
            ("аirdrор", true), // Cyrillic а and Cyrillic р
            ("aіrdrop", true), // Cyrillic і
            ("аіrdrop", true), // Cyrillic а and Cyrillic і
            ("аіrdrop", true), // Cyrillic а and Cyrillic і
            ("аіrdrop", true), // Cyrillic а and Cyrillic і
            ("aіrdroр", true), // Cyrillic і and Cyrillic р
            ("аіrdroр", true), // Cyrillic а, Cyrillic і, and Cyrillic р
            ("аirdrор", true), // Cyrillic а and Cyrillic р
            ("aіrԁrор", true), // Cyrillic і, Cyrillic ԁ, and Cyrillic р
            ("aіrԁrop", true), // Cyrillic і and Cyrillic ԁ
            ("airdroр", true), // Cyrillic р
            ("airԁrop", true), // Greek delta, Δ
            ("аirdrор", true), // Greek o, ο
            ("аіrԁrop", true), // Greek iota, ι
            ("airԁroр", true), // Greek rho, ρ
            ("аirdrор", true), // Greek omicron, ο
            ("aіrdrop", true), // Greek iota, ι
            ("аіrdrop", true), // Greek alpha, α
            ("аіrdrop", true), // Greek iota, ι
            ("aіrdroр", true), // Greek iota, ι, and rho, ρ
            ("аirdrор", true), // Greek omicron, ο, and rho, ρ
            ("aіrԁrор", true), // Greek iota, ι, delta, Δ, and rho, ρ
            ("aіrԁrop", true), // Greek iota, ι, and delta, Δ
            ("airdroр", true), // Greek rho, ρ
            ("Сlаim  Q СOMMUNITY АIRDROP\n Join the Q movement.", true), // snippet from a real one
            ("🅰irdrop", true), // with emoji
            ("🅰️ℹ️irdr🅾️🇵", true), // with emojis
            ("air drop", true), // with space
            ("a i r d r o p", true), // with single spaces
            ("a i r d r o p", true), // with different kids of spaces
            ("🇦 🇮 🇷 🇩 🇷 🇴 🇵", true), // with special characters and spaces
            ("42", false),
            ("", false),
            ("token", false),
            ("wallet", false),
            ("get your wallet and fill it with free tokens", true),
            ("win many tokens for your new wallet", true),
            ("ask me how to get free tokens\n\nfor your wallet", true),
            ("My walleТs have tokens", true),
        ];
        for (word, expected) in test_cases {
            for w in [word, word.to_uppercase().as_str()] {
                let model = RegularExpression::new().await.unwrap();
                let got = model.is_spam(w).await.unwrap();
                assert_eq!(
                    got, expected,
                    "expected: {:?} for {:?}, got: {:?}",
                    expected, w, got
                );
            }
        }
    }
}
