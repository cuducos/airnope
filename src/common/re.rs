use crate::{truncated, Guess};
use anyhow::Result;
use regex::{Regex, RegexBuilder};

const A: &str = "[аa🅰🅰️🇦🇦о]";
const B: &str = "[bB🇧]";
const C: &str = "[cC]";
const D: &str = "[dԁ🇩]";
const E: &str = "[eEе3€ℯ🇪]";
const F: &str = "[fF🇫]";
const G: &str = "[gG9🇬]";
const I: &str = "[іiíÍI1lℹ️🇮]";
const K: &str = "[kK🇰]";
const L: &str = "[lL1|ℓ🇱]";
const M: &str = "[mM]";
const N: &str = "[nNℕñÑ🇳]";
const O: &str = "[оo0🅾️🇴]";
const P: &str = "[рpρϱ🅿️🇵]";
const Q: &str = "[qQ9🇶]";
const R: &str = "[рr🇷]";
const S: &str = "[sSЅ]";
const T: &str = "[tTТ7†🇹]";
const U: &str = "[uUµ🇺]";
const V: &str = "[vV]";
const W: &str = "[wW🇼]";
const Y: &str = "[yY¥🇾]";

#[derive(Clone)]
pub struct RegularExpression {
    airdrop: Regex,
    cryptocurrency: Regex,
    altcoin: Regex,
    wallet: Regex,
    token: Regex,
    claim: Regex,
    swap: Regex,
    reward: Regex,
    crypto: Regex,
    opportunity: Regex,
    finance: Regex,
    network: Regex,
    ganar: Regex,
    invertido: Regex,
    clic: Regex,
    aqui: Regex,
    bitcoin: Regex,
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
        let altcoin = to_regex([A, L, T, C, O, I, N])?;
        let cryptocurrency = to_regex([C, R, Y, P, T, O, C, U, R, R, E, N, C, Y])?;
        let wallet = to_regex([W, A, L, L, E, T])?;
        let token = to_regex([T, O, K, E, N])?;
        let claim = to_regex([C, L, A, I, M])?;
        let swap = to_regex([S, W, A, P])?;
        let reward = to_regex([R, E, W, A, R, D])?;
        let crypto = to_regex([C, R, Y, P, T, O])?;
        let opportunity = to_regex([O, P, P, O, R, T, U, N, I, T, Y])?;
        let finance = to_regex([F, I, N, A, N, C, E])?;
        let network = to_regex([N, E, T, W, O, R, K])?;
        let ganar = to_regex([G, A, N, A, R])?;
        let invertido = to_regex([I, N, V, E, R, T, I, D, O])?;
        let clic = to_regex([C, L, I, C])?;
        let aqui = to_regex([A, Q, U, I])?;
        let bitcoin = to_regex([B, I, T, C, O, I, N])?;
        let cleanup = Regex::new(r"\s")?;
        Ok(Self {
            airdrop,
            cryptocurrency,
            altcoin,
            wallet,
            token,
            claim,
            swap,
            reward,
            crypto,
            opportunity,
            finance,
            network,
            ganar,
            invertido,
            clic,
            aqui,
            bitcoin,
            cleanup,
        })
    }

    pub async fn is_spam(&self, txt: &str) -> Result<Guess> {
        let cleaned = self.cleanup.replace_all(txt, " ");
        let result = self.airdrop.is_match(&cleaned)
            || self.cryptocurrency.is_match(&cleaned)
            || self.altcoin.is_match(&cleaned)
            || (self.wallet.is_match(&cleaned) && self.token.is_match(&cleaned))
            || (self.wallet.is_match(&cleaned) && self.reward.is_match(&cleaned))
            || (self.token.is_match(&cleaned) && self.network.is_match(&cleaned))
            || (self.claim.is_match(&cleaned) && self.swap.is_match(&cleaned))
            || (self.crypto.is_match(&cleaned) && self.reward.is_match(&cleaned))
            || (self.crypto.is_match(&cleaned) && self.opportunity.is_match(&cleaned))
            || (self.finance.is_match(&cleaned) && self.reward.is_match(&cleaned))
            || (self.finance.is_match(&cleaned) && self.network.is_match(&cleaned))
            || (self.ganar.is_match(&cleaned)
                && self.invertido.is_match(&cleaned)
                && self.clic.is_match(&cleaned)
                && self.aqui.is_match(&cleaned))
            || (self.ganar.is_match(&cleaned) && self.bitcoin.is_match(&cleaned));
        if result {
            log::info!("Message detected as spam by RegularExpression");
            log::debug!("{}", truncated(txt));
        }
        Ok(Guess {
            is_spam: result,
            score: None,
            scores: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{fs, io::AsyncReadExt};

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
            ("wallet tokens", true),
            ("tokens wallet", true),
            ("wallеt and tokеn", true), // with Cyrillic е
        ];
        for (word, expected) in test_cases {
            for w in [word, word.to_uppercase().as_str()] {
                let model = RegularExpression::new().await.unwrap();
                let got = model.is_spam(w).await.unwrap();
                assert_eq!(
                    got.is_spam, expected,
                    "expected: {:?} for {:?}, got: {:?}",
                    expected, w, got.is_spam
                );
                assert_eq!(got.score, None);
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_is_spam_with_test_data() {
        let model = RegularExpression::new().await.unwrap();
        let mut entries = fs::read_dir("test_data").await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            let path = entry.path();
            let mut contents = String::new();
            let mut file = fs::File::open(&path).await.unwrap();
            file.read_to_string(&mut contents).await.unwrap();
            let got = model.is_spam(contents.as_str()).await.unwrap();
            assert!(got.is_spam, "{} was not flagged as spam", path.display(),);
        }
    }
}
