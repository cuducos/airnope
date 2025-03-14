use crate::{truncated, Guess};
use anyhow::Result;
use regex::{Regex, RegexBuilder};

const A: &str = "[аaã🅰🅰️🇦🇦о]";
const B: &str = "[bB🇧]";
const C: &str = "[cç]";
const D: &str = "[dԁ🇩]";
const E: &str = "[eEе3€ℯ🇪]";
const F: &str = "[fF🇫]";
const G: &str = "[gG9🇬]";
const H: &str = "[hH🇭]";
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
const Z: &str = "[zZ2Ζ🇿]";

#[derive(Clone)]
pub struct RegularExpression {
    // generic
    airdrop: Regex,
    bitcoin: Regex,
    btc: Regex,
    altcoin: Regex,
    crypto: Regex,
    https: Regex,
    safeguard: Regex,

    // english
    cryptocurrenc: Regex,
    wallet: Regex,
    token: Regex,
    claim: Regex,
    swap: Regex,
    reward: Regex,
    opportunity: Regex,
    finance: Regex,
    network: Regex,
    contract: Regex,
    fund: Regex,
    transaction: Regex,
    trading: Regex,
    trade: Regex,

    // spanish
    ganar: Regex,     // win, receiving
    invertido: Regex, // invested
    clic: Regex,      // click
    aqui: Regex,      // here

    // portuguese
    plataforma: Regex,   // platform
    distribuicao: Regex, // distribution
    paga: Regex,         // paid

    // german
    plattform: Regex,   // platform
    gewinne: Regex,     // profits
    eingezahlt: Regex,  // deposited
    erhalten: Regex,    // received
    investieren: Regex, // investing

    dollar_word: Regex,
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
        let bitcoin = to_regex([B, I, T, C, O, I, N])?;
        let btc = to_regex([B, T, C])?;
        let altcoin = to_regex([A, L, T, C, O, I, N])?;
        let crypto = to_regex([C, R, Y, P, T, O])?;
        let https = to_regex([H, T, T, P, S])?;
        let safeguard = to_regex([S, A, F, E, G, U, A, R, D])?;
        let cryptocurrenc = to_regex([C, R, Y, P, T, O, C, U, R, R, E, N, C])?;
        let wallet = to_regex([W, A, L, L, E, T])?;
        let token = to_regex([T, O, K, E, N])?;
        let claim = to_regex([C, L, A, I, M])?;
        let swap = to_regex([S, W, A, P])?;
        let reward = to_regex([R, E, W, A, R, D])?;
        let opportunity = to_regex([O, P, P, O, R, T, U, N, I, T])?;
        let finance = to_regex([F, I, N, A, N, C, E])?;
        let network = to_regex([N, E, T, W, O, R, K])?;
        let contract = to_regex([C, O, N, T, R, A, C, T])?;
        let fund = to_regex([F, U, N, D])?;
        let transaction = to_regex([T, R, A, N, S, A, C, T, I, O, N])?;
        let trading = to_regex([T, R, A, D, I, N, G])?;
        let trade = to_regex([T, R, A, D, E])?;
        let ganar = to_regex([G, A, N, A, R])?;
        let invertido = to_regex([I, N, V, E, R, T, I, D, O])?;
        let clic = to_regex([C, L, I, C])?;
        let aqui = to_regex([A, Q, U, I])?;
        let plataforma = to_regex([P, L, A, T, A, F, O, R, M, A])?;
        let distribuicao = to_regex([D, I, S, T, R, I, B, U, I, C, A, O])?;
        let paga = to_regex([P, A, G, A])?;
        let plattform = to_regex([P, L, A, T, T, F, O, R, M])?;
        let gewinne = to_regex([G, E, W, I, N, N, E])?;
        let eingezahlt = to_regex([E, I, N, G, E, Z, A, H, L, T])?;
        let erhalten = to_regex([E, R, H, A, L, T, E, N])?;
        let investieren = to_regex([I, N, V, E, S, T, I, E, R, E, N])?;
        let dollar_word = Regex::new(r"\$\w+")?;
        let cleanup = Regex::new(r"\s")?;
        Ok(Self {
            airdrop,
            bitcoin,
            btc,
            altcoin,
            crypto,
            https,
            safeguard,
            cryptocurrenc,
            wallet,
            token,
            claim,
            swap,
            reward,
            opportunity,
            finance,
            network,
            contract,
            fund,
            transaction,
            trading,
            trade,
            ganar,
            invertido,
            clic,
            aqui,
            plataforma,
            distribuicao,
            paga,
            plattform,
            gewinne,
            eingezahlt,
            erhalten,
            investieren,
            dollar_word,
            cleanup,
        })
    }

    pub async fn is_spam(&self, txt: &str) -> Result<Guess> {
        let cleaned = self.cleanup.replace_all(txt, " ");
        let result = self.airdrop.is_match(&cleaned)
            || self.cryptocurrenc.is_match(&cleaned)
            || self.altcoin.is_match(&cleaned)
            || self.safeguard.is_match(&cleaned)
            || (self.wallet.is_match(&cleaned) && self.token.is_match(&cleaned))
            || (self.wallet.is_match(&cleaned) && self.reward.is_match(&cleaned))
            || (self.wallet.is_match(&cleaned) && self.dollar_word.is_match(&cleaned))
            || (self.token.is_match(&cleaned) && self.network.is_match(&cleaned))
            || (self.token.is_match(&cleaned) && self.contract.is_match(&cleaned))
            || (self.token.is_match(&cleaned) && self.fund.is_match(&cleaned))
            || (self.claim.is_match(&cleaned) && self.swap.is_match(&cleaned))
            || (self.claim.is_match(&cleaned) && self.token.is_match(&cleaned))
            || (self.crypto.is_match(&cleaned) && self.reward.is_match(&cleaned))
            || (self.crypto.is_match(&cleaned) && self.opportunity.is_match(&cleaned))
            || (self.finance.is_match(&cleaned) && self.reward.is_match(&cleaned))
            || (self.finance.is_match(&cleaned) && self.network.is_match(&cleaned))
            || (self.transaction.is_match(&cleaned) && self.trading.is_match(&cleaned))
            || (self.transaction.is_match(&cleaned) && self.trade.is_match(&cleaned))
            || (self.ganar.is_match(&cleaned)
                && self.invertido.is_match(&cleaned)
                && self.clic.is_match(&cleaned)
                && self.aqui.is_match(&cleaned))
            || (self.ganar.is_match(&cleaned) && self.bitcoin.is_match(&cleaned))
            || (self.bitcoin.is_match(&cleaned) && self.https.is_match(&cleaned))
            || (self.btc.is_match(&cleaned) && self.https.is_match(&cleaned))
            || (self.plataforma.is_match(&cleaned)
                && self.distribuicao.is_match(&cleaned)
                && self.paga.is_match(&cleaned))
            || (self.plattform.is_match(&cleaned) && self.gewinne.is_match(&cleaned))
            || (self.plattform.is_match(&cleaned) && self.eingezahlt.is_match(&cleaned))
            || (self.plattform.is_match(&cleaned) && self.erhalten.is_match(&cleaned))
            || (self.plattform.is_match(&cleaned) && self.investieren.is_match(&cleaned));
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
