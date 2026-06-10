use crate::{truncated, Guess};
use anyhow::Result;
use regex::{Regex, RegexBuilder};

#[derive(Clone)]
pub struct RegularExpression {
    // generic
    airdrop: Regex,
    bitcoin: Regex,
    btc: Regex,
    altcoin: Regex,
    crypto: Regex,
    https: Regex,
    nft: Regex,
    safeguard: Regex,
    somnia: Regex,
    cvv: Regex,
    bet: Regex,

    // english
    cryptocurrenc: Regex,
    wallet: Regex,
    token: Regex,
    claim: Regex,
    swap: Regex,
    reward: Regex,
    earning: Regex,
    opportunity: Regex,
    finance: Regex,
    network: Regex,
    contract: Regex,
    fund: Regex,
    transaction: Regex,
    trading: Regex,
    trade: Regex,
    platform: Regex,
    drop: Regex,

    // spanish
    gana: Regex,        // win, receiving
    inverti: Regex,     // invested
    fondo: Regex,       // fund
    cuenta: Regex,      // account
    clic: Regex,        // click
    aqui: Regex,        // here
    criptomoned: Regex, // cryptocurrency
    ingreso: Regex,     // income

    // portuguese
    plataforma: Regex,   // platform
    distribuicao: Regex, // distribution
    paga: Regex,         // paid
    conta: Regex,        // account
    aposta: Regex,       // bet
    cartao: Regex,       // card
    saldo: Regex,        // balance
    garant: Regex,       // guarantee (garantia, garanto)
    risco: Regex,        // risk

    // german
    plattform: Regex,   // platform
    gewinne: Regex,     // profits
    eingezahlt: Regex,  // deposited
    erhalten: Regex,    // received
    investieren: Regex, // investing
    auszahlung: Regex,  // payout
    belohn: Regex,      // reward
    verdien: Regex,     // earned
    handel: Regex,      // traded

    dollar_word: Regex,
    cleanup: Regex,
}

fn to_regex(word: &str) -> Result<Regex> {
    let pattern = word
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(r"\s?");
    Ok(RegexBuilder::new(&pattern).case_insensitive(true).build()?)
}

impl RegularExpression {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            airdrop: to_regex("airdrop")?,
            bitcoin: to_regex("bitcoin")?,
            btc: to_regex("btc")?,
            altcoin: to_regex("altcoin")?,
            crypto: to_regex("crypto")?,
            https: to_regex("https")?,
            safeguard: to_regex("safeguard")?,
            somnia: to_regex("somnia")?,
            cvv: to_regex("cvv")?,
            bet: to_regex("bet")?,
            nft: to_regex("nft")?,
            cryptocurrenc: to_regex("cryptocurrenc")?,
            wallet: to_regex("wallet")?,
            token: to_regex("token")?,
            claim: to_regex("claim")?,
            swap: to_regex("swap")?,
            reward: to_regex("reward")?,
            earning: to_regex("earning")?,
            opportunity: to_regex("opportunit")?,
            finance: to_regex("finance")?,
            network: to_regex("network")?,
            contract: to_regex("contract")?,
            fund: to_regex("fund")?,
            transaction: to_regex("transaction")?,
            trading: to_regex("trading")?,
            trade: to_regex("trade")?,
            platform: to_regex("platform")?,
            drop: to_regex("drop")?,
            gana: to_regex("gana")?,
            inverti: to_regex("inverti")?,
            fondo: to_regex("fondo")?,
            cuenta: to_regex("cuenta")?,
            clic: to_regex("clic")?,
            aqui: to_regex("aqui")?,
            criptomoned: to_regex("criptomoned")?,
            ingreso: to_regex("ingreso")?,
            plataforma: to_regex("plataforma")?,
            distribuicao: to_regex("distribuicao")?,
            paga: to_regex("paga")?,
            conta: to_regex("conta")?,
            aposta: to_regex("aposta")?,
            cartao: to_regex("cartao")?,
            saldo: to_regex("saldo")?,
            garant: to_regex("garant")?,
            risco: to_regex("risco")?,
            plattform: to_regex("plattform")?,
            gewinne: to_regex("gewinne")?,
            eingezahlt: to_regex("eingezahlt")?,
            erhalten: to_regex("erhalten")?,
            investieren: to_regex("investieren")?,
            auszahlung: to_regex("auszahlung")?,
            belohn: to_regex("belohn")?,
            verdien: to_regex("verdien")?,
            handel: to_regex("handel")?,
            dollar_word: Regex::new(r"\$\w+")?,
            cleanup: Regex::new(r"\s")?,
        })
    }

    pub async fn is_spam(&self, txt: &str) -> Result<Guess> {
        let cleaned = self.cleanup.replace_all(txt, " ");
        let result = self.airdrop.is_match(&cleaned)
            || self.cryptocurrenc.is_match(&cleaned)
            || self.altcoin.is_match(&cleaned)
            || self.safeguard.is_match(&cleaned)
            || self.somnia.is_match(&cleaned)
            || (self.cvv.is_match(&cleaned) && self.garant.is_match(&cleaned))
            || (self.cartao.is_match(&cleaned) && self.garant.is_match(&cleaned))
            || (self.saldo.is_match(&cleaned) && self.garant.is_match(&cleaned))
            || (self.wallet.is_match(&cleaned) && self.token.is_match(&cleaned))
            || (self.wallet.is_match(&cleaned) && self.reward.is_match(&cleaned))
            || (self.wallet.is_match(&cleaned) && self.swap.is_match(&cleaned))
            || (self.wallet.is_match(&cleaned) && self.dollar_word.is_match(&cleaned))
            || (self.wallet.is_match(&cleaned) && self.nft.is_match(&cleaned))
            || (self.network.is_match(&cleaned) && self.nft.is_match(&cleaned))
            || (self.platform.is_match(&cleaned) && self.nft.is_match(&cleaned))
            || (self.platform.is_match(&cleaned)
                && self.trade.is_match(&cleaned)
                && self.https.is_match(&cleaned))
            || (self.token.is_match(&cleaned) && self.network.is_match(&cleaned))
            || (self.token.is_match(&cleaned) && self.contract.is_match(&cleaned))
            || (self.token.is_match(&cleaned) && self.fund.is_match(&cleaned))
            || (self.claim.is_match(&cleaned) && self.swap.is_match(&cleaned))
            || (self.claim.is_match(&cleaned) && self.token.is_match(&cleaned))
            || (self.crypto.is_match(&cleaned) && self.reward.is_match(&cleaned))
            || (self.crypto.is_match(&cleaned) && self.opportunity.is_match(&cleaned))
            || (self.crypto.is_match(&cleaned) && self.earning.is_match(&cleaned))
            || (self.finance.is_match(&cleaned) && self.reward.is_match(&cleaned))
            || (self.finance.is_match(&cleaned) && self.network.is_match(&cleaned))
            || (self.transaction.is_match(&cleaned) && self.trading.is_match(&cleaned))
            || (self.transaction.is_match(&cleaned) && self.trade.is_match(&cleaned))
            || (self.gana.is_match(&cleaned)
                && self.inverti.is_match(&cleaned)
                && self.clic.is_match(&cleaned)
                && self.aqui.is_match(&cleaned))
            || (self.inverti.is_match(&cleaned) && self.fondo.is_match(&cleaned))
            || (self.inverti.is_match(&cleaned) && self.cuenta.is_match(&cleaned))
            || (self.criptomoned.is_match(&cleaned) && self.ingreso.is_match(&cleaned))
            || (self.gana.is_match(&cleaned) && self.bitcoin.is_match(&cleaned))
            || (self.gana.is_match(&cleaned) && self.trading.is_match(&cleaned))
            || (self.bitcoin.is_match(&cleaned) && self.https.is_match(&cleaned))
            || (self.btc.is_match(&cleaned) && self.https.is_match(&cleaned))
            || (self.plataforma.is_match(&cleaned)
                && self.distribuicao.is_match(&cleaned)
                && self.paga.is_match(&cleaned))
            || (self.bet.is_match(&cleaned) && self.conta.is_match(&cleaned))
            || (self.aposta.is_match(&cleaned) && self.conta.is_match(&cleaned))
            || (self.bet.is_match(&cleaned) && self.risco.is_match(&cleaned))
            || (self.aposta.is_match(&cleaned) && self.risco.is_match(&cleaned))
            || (self.plattform.is_match(&cleaned) && self.gewinne.is_match(&cleaned))
            || (self.plattform.is_match(&cleaned) && self.eingezahlt.is_match(&cleaned))
            || (self.plattform.is_match(&cleaned) && self.erhalten.is_match(&cleaned))
            || (self.plattform.is_match(&cleaned) && self.investieren.is_match(&cleaned))
            || (self.auszahlung.is_match(&cleaned) && self.belohn.is_match(&cleaned))
            || (self.verdien.is_match(&cleaned) && self.handel.is_match(&cleaned))
            || (self.drop.is_match(&cleaned)
                && self.network.is_match(&cleaned)
                && self.claim.is_match(&cleaned));

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
    use crate::normalize;
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
            ("🅰️ℹ️rdr🅾️🇵", true), // with emojis
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
                let cleansed = normalize(w);
                let got = model.is_spam(&cleansed).await.unwrap();
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
            if path.extension().unwrap() != "txt" {
                continue;
            }
            let mut contents = String::new();
            let mut file = fs::File::open(&path).await.unwrap();
            file.read_to_string(&mut contents).await.unwrap();
            let cleansed = normalize(contents.as_str());
            let got = model.is_spam(&cleansed).await.unwrap();
            assert!(got.is_spam, "{} was not flagged as spam", path.display(),);
        }
    }
}
