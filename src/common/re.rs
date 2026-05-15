use crate::{truncated, Guess};
use anyhow::Result;
use regex::{Regex, RegexBuilder};

const A: &str = "[аaã🅰🅰️🇦🇦о𝐀𝐚𝐴𝑎𝑨𝒂𝖠𝖺𝗔𝗮𝙰𝚊]";
const B: &str = "[bB🇧𝐁𝐛𝐵𝑏𝑩𝒃𝖡𝖻𝗕𝗯𝙱𝚋]";
const C: &str = "[cç𝐂𝐜𝐶𝑐𝑪𝒄𝖢𝖼𝗖𝗰𝙲𝚌]";
const D: &str = "[dԁ🇩𝐃𝐝𝐷𝑑𝑫𝒅𝖣𝖽𝗗𝗱𝙳𝚍]";
const E: &str = "[eEе3€ℯ🇪𝐄𝐞𝐸𝑒𝑬𝒆𝖤𝖾𝗘𝗲𝙴𝚎]";
const F: &str = "[fF🇫𝐅𝐟𝐹𝑓𝑭𝒇𝖥𝖿𝗙𝗳𝙵𝚏]";
const G: &str = "[gG9🇬𝐆𝐠𝐺𝑔𝑮𝒈𝖦𝗀𝗚𝗴𝙶𝚐]";
const H: &str = "[hH🇭𝐇𝐡𝐻ℎ𝑯𝒉𝖧𝗁𝗛𝗵𝙷𝚑]";
const I: &str = "[іiíÍI1lℹ️🇮𝐈𝐢𝐼𝑖𝑰𝒊𝖨𝗂𝗜𝗶𝙸𝚒]";
const K: &str = "[kK🇰𝐊𝐤𝐾𝑘𝑲𝒌𝖪𝗄𝗞𝗸𝙺𝚔]";
const L: &str = "[lL1|ℓ🇱𝐋𝐥𝐿𝑙𝑳𝒍𝖫𝗅𝗟𝗹𝙻𝚕]";
const M: &str = "[mM𝐌𝐦𝑀𝑚𝑴𝒎𝖬𝗆𝗠𝗺𝙼𝚖]";
const N: &str = "[nNℕñÑ🇳𝐍𝐧𝑁𝑛𝑵𝒏𝖭𝗇𝗡𝗻𝙽𝚗]";
const O: &str = "[оo0🅾️🇴𝐎𝐨𝑂𝑜𝑶𝒐𝖮𝗈𝗢𝗼𝙾𝚘]";
const P: &str = "[рpρϱ🅿️🇵𝐏𝐩𝑃𝑝𝑷𝒑𝖯𝗉𝗣𝗽𝙿𝚙]";
const Q: &str = "[qQ9🇶𝐐𝐪𝑄𝑞𝑸𝒒𝖰𝗊𝗤𝗾𝚀𝚚]";
const R: &str = "[рr🇷𝐑𝐫𝑅𝑟𝑹𝒓𝖱𝗋𝗥𝗿𝚁𝚛]";
const S: &str = "[sSЅ𝐒𝐬𝑆𝑠𝑺𝒔𝖲𝗌𝗦𝘀𝚂𝚜]";
const T: &str = "[tTТ7†🇹𝐓𝐭𝑇𝑡𝑻𝒕𝖳𝗍𝗧𝘁𝚃𝚝]";
const U: &str = "[uUµ🇺𝐔𝐮𝑈𝑢𝑼𝒖𝖴𝗎𝗨𝘂𝚄𝚞]";
const V: &str = "[vV𝐕𝐯𝑉𝑣𝑽𝒗𝖵𝗏𝗩𝘃𝚅𝚟]";
const W: &str = "[wW🇼𝐖𝐰𝑊𝑤𝑾𝒘𝖶𝗐𝗪𝘄𝚆𝚠]";
const Y: &str = "[yY¥🇾𝐘𝐲𝑌𝑦𝒀𝒚𝖸𝗒𝗬𝘆𝚈𝚢]";
const Z: &str = "[zZ2Ζ🇿𝐙𝐳𝑍𝑧𝒁𝒛𝖹𝗓𝗭𝘇𝚉𝚣]";

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
        let somnia = to_regex([S, O, M, N, I, A])?;
        let cvv = to_regex([C, V, V])?;
        let bet = to_regex([B, E, T])?;
        let nft = to_regex([N, F, T])?;
        let cryptocurrenc = to_regex([C, R, Y, P, T, O, C, U, R, R, E, N, C])?;
        let wallet = to_regex([W, A, L, L, E, T])?;
        let token = to_regex([T, O, K, E, N])?;
        let claim = to_regex([C, L, A, I, M])?;
        let swap = to_regex([S, W, A, P])?;
        let reward = to_regex([R, E, W, A, R, D])?;
        let earning = to_regex([E, A, R, N, I, N, G])?;
        let opportunity = to_regex([O, P, P, O, R, T, U, N, I, T])?;
        let finance = to_regex([F, I, N, A, N, C, E])?;
        let network = to_regex([N, E, T, W, O, R, K])?;
        let contract = to_regex([C, O, N, T, R, A, C, T])?;
        let fund = to_regex([F, U, N, D])?;
        let transaction = to_regex([T, R, A, N, S, A, C, T, I, O, N])?;
        let trading = to_regex([T, R, A, D, I, N, G])?;
        let trade = to_regex([T, R, A, D, E])?;
        let platform = to_regex([P, L, A, T, F, O, R, M])?;
        let drop = to_regex([D, R, O, P])?;
        let gana = to_regex([G, A, N, A])?;
        let inverti = to_regex([I, N, V, E, R, T, I])?;
        let fondo = to_regex([F, O, N, D, O])?;
        let cuenta = to_regex([C, U, E, N, T, A])?;
        let clic = to_regex([C, L, I, C])?;
        let aqui = to_regex([A, Q, U, I])?;
        let criptomoned = to_regex([C, R, I, P, T, O, M, O, N, E, D])?;
        let ingreso = to_regex([I, N, G, R, E, S, O])?;
        let plataforma = to_regex([P, L, A, T, A, F, O, R, M, A])?;
        let distribuicao = to_regex([D, I, S, T, R, I, B, U, I, C, A, O])?;
        let paga = to_regex([P, A, G, A])?;
        let conta = to_regex([C, O, N, T, A])?;
        let aposta = to_regex([A, P, O, S, T, A])?;
        let cartao = to_regex([C, A, R, T, A, O])?;
        let saldo = to_regex([S, A, L, D, O])?;
        let garant = to_regex([G, A, R, A, N, T])?;
        let risco = to_regex([R, I, S, C, O])?;
        let plattform = to_regex([P, L, A, T, T, F, O, R, M])?;
        let gewinne = to_regex([G, E, W, I, N, N, E])?;
        let eingezahlt = to_regex([E, I, N, G, E, Z, A, H, L, T])?;
        let erhalten = to_regex([E, R, H, A, L, T, E, N])?;
        let investieren = to_regex([I, N, V, E, S, T, I, E, R, E, N])?;
        let auszahlung = to_regex([A, U, S, Z, A, H, L, U, N, G])?;
        let belohn = to_regex([B, E, L, O, H, N])?;
        let verdien = to_regex([V, E, R, D, I, E, N])?;
        let handel = to_regex([H, A, N, D, E, L])?;
        let dollar_word = Regex::new(r"\$\w+")?;
        let cleanup = Regex::new(r"\s")?;
        Ok(Self {
            airdrop,
            bitcoin,
            btc,
            altcoin,
            crypto,
            https,
            nft,
            safeguard,
            somnia,
            cvv,
            bet,
            cryptocurrenc,
            wallet,
            token,
            claim,
            swap,
            reward,
            earning,
            opportunity,
            finance,
            network,
            contract,
            fund,
            transaction,
            trading,
            trade,
            platform,
            drop,
            gana,
            inverti,
            fondo,
            cuenta,
            clic,
            aqui,
            criptomoned,
            ingreso,
            plataforma,
            distribuicao,
            paga,
            conta,
            aposta,
            cartao,
            saldo,
            garant,
            risco,
            plattform,
            gewinne,
            eingezahlt,
            erhalten,
            investieren,
            auszahlung,
            belohn,
            verdien,
            handel,
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
            if path.extension().unwrap() != "txt" {
                continue;
            }
            let mut contents = String::new();
            let mut file = fs::File::open(&path).await.unwrap();
            file.read_to_string(&mut contents).await.unwrap();
            let got = model.is_spam(contents.as_str()).await.unwrap();
            assert!(got.is_spam, "{} was not flagged as spam", path.display(),);
        }
    }
}
