pub mod common;
pub use common::embeddings;
pub use common::re;
pub use common::telegram;
pub use common::zsc;

use anyhow::Result;
use common::zsc::ZeroShotClassification;
use std::sync::Arc;
use tokio::sync::Mutex;
use unicode_normalization::UnicodeNormalization;

const MESSAGE_PREVIEW_SIZE: usize = 128;
const A: &str = "аaã🅰🅰️🇦🇦𝐀𝐚𝐴𝑎𝑨𝒂𝖠𝖺𝗔𝗮𝙰α𝙰";
const B: &str = "🇧𝐁𝐛𝐵𝑏𝑩𝒃𝖡𝖻𝗕𝗯B𝚋𝙱";
const C: &str = "ç𝐂𝐜𝐶𝑐𝑪𝒄𝖢𝖼𝗖𝗰Cｃ𝙲";
const D: &str = "ԁ🇩𝐃𝐝𝐷𝑑𝑫𝒅𝖣𝖽𝗗𝗱D𝚍𝙳";
const E: &str = "е3€ℯ🇪𝐄𝐞𝐸𝑒𝑬𝒆𝖤𝖾𝗘𝗲Eｅ𝙴";
const F: &str = "🇫𝐅𝐟𝐹𝑓𝑭𝒇𝖥𝖿𝗙𝗳F𝚏F";
const G: &str = "9🇬𝐆𝐠𝐺𝑔𝑮𝒈𝖦𝗀𝗚𝗴G𝚐𝙶";
const H: &str = "🇭𝐇𝐡HH𝑯𝒉𝖧𝗁𝗛𝗵H𝚑𝙷";
const I: &str = "іíÍI1ℹ️🇮𝐈𝐢𝐼𝑖𝑰𝒊𝖨𝗂𝗜𝗶Ii𝙸";
const K: &str = "kK🇰𝐊𝐤KK𝑲𝒌𝖪𝗄𝗞𝗸K𝚔";
const L: &str = "|ℓ🇱𝐋𝐥𝐿𝑙𝑳𝒍𝖫𝗅🇱𝗹𝙻";
const M: &str = "mM𝐌𝐦𝑀𝑚𝑴𝒎𝖬𝗆𝗠𝒎Mm𝗺𝙼";
const N: &str = "nNℕñÑ🇳𝐍𝐧NN𝑵𝒏𝖭𝗇ＮｎNn𝙽";
const O: &str = "оo0🅾️🇴𝐎𝐨OO𝑶𝒐𝖮𝗈𝗢𝗼Ooо𝙾";
const P: &str = "рpρϱ🅿️🇵𝐏𝐩PP𝑷𝒑𝖯𝗉𝗣𝒑Ppр𝙿";
const Q: &str = "qQ🇶𝐐𝐪QQ𝑞𝑸𝒒𝖰𝗊𝗤𝗾Qq𝚀";
const R: &str = "r🇷𝐑𝐫RR𝑹𝒓𝖱𝗋𝗥𝗿Rr𝚁";
const S: &str = "sSЅ𝐒𝐬SS𝑺𝒔𝖲𝗌𝗦𝘀Ss𝚂";
const T: &str = "tTТ7†🇹𝐓𝐭TT𝑻𝒕𝖳𝗍Ｔ𝘁Tt𝚃";
const U: &str = "uUµ🇺𝐔𝐮UU𝑼𝒖𝖴𝗎𝗨𝒖Uu𝘂𝚄";
const V: &str = "vV𝐕𝐯VV𝑽𝒗𝖵𝗏𝗩𝘃Vv𝚅";
const W: &str = "wW🇼𝐖𝐰WW𝑾𝒘𝖶𝗐𝗪𝘄Ww";
const X: &str = "𝚇";
const Y: &str = "yY¥🇾𝐘𝐲YY𝒀𝒚𝖸𝗒𝗬𝘆Yy";
const Z: &str = "zZ2Ζ🇿𝐙𝐳ZZ𝒁𝒛𝖹𝗓𝗭𝘇Zz";

#[derive(Debug, PartialEq, Clone)]
pub struct Guess {
    pub is_spam: bool,
    pub score: Option<f32>,
    pub scores: Vec<f32>,
}

fn is_combining_mark(c: char) -> bool {
    (0x300..=0x36F).contains(&(c as u32))
}

pub fn normalize(text: &str) -> String {
    text.to_lowercase()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .filter(|&c| c != '\u{fe0f}' && c != '\u{200d}' && c != '\u{200c}')
        .map(|c| {
            if A.contains(c) {
                'a'
            } else if B.contains(c) {
                'b'
            } else if C.contains(c) {
                'c'
            } else if D.contains(c) {
                'd'
            } else if E.contains(c) {
                'e'
            } else if F.contains(c) {
                'f'
            } else if G.contains(c) {
                'g'
            } else if H.contains(c) {
                'h'
            } else if I.contains(c) {
                'i'
            } else if K.contains(c) {
                'k'
            } else if L.contains(c) {
                'l'
            } else if M.contains(c) {
                'm'
            } else if N.contains(c) {
                'n'
            } else if O.contains(c) {
                'o'
            } else if P.contains(c) {
                'p'
            } else if Q.contains(c) {
                'q'
            } else if R.contains(c) {
                'r'
            } else if S.contains(c) {
                's'
            } else if T.contains(c) {
                't'
            } else if U.contains(c) {
                'u'
            } else if V.contains(c) {
                'v'
            } else if W.contains(c) {
                'w'
            } else if X.contains(c) {
                'x'
            } else if Y.contains(c) {
                'y'
            } else if Z.contains(c) {
                'z'
            } else {
                c
            }
        })
        .collect()
}

pub async fn is_spam_with_custom_classifier(
    embeddings: &Arc<Mutex<embeddings::Embeddings>>,
    classifier: ZeroShotClassification,
    txt: &str,
) -> Result<Guess> {
    let normalized = normalize(txt);
    let result = re::RE.is_spam(&normalized)?;
    if !result.is_spam {
        return Ok(result);
    }
    classifier.is_spam(embeddings, txt).await // Must use original text for anomaly detector to see homoglyphs
}

pub async fn is_spam(embeddings: &Arc<Mutex<embeddings::Embeddings>>, txt: &str) -> Result<Guess> {
    let zero_shot = zsc::ZeroShotClassification::default(embeddings).await?;
    is_spam_with_custom_classifier(embeddings, zero_shot, txt).await
}

fn truncated(message: &str) -> String {
    let mut msg = message.to_string();
    msg.retain(|c| !c.is_control() || c == ' ');
    msg = msg.trim().to_string();
    if msg.chars().count() > MESSAGE_PREVIEW_SIZE {
        msg = msg
            .chars()
            .take(MESSAGE_PREVIEW_SIZE - 3)
            .collect::<String>();
        msg.push_str("...");
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Read;
    use std::path::Path;
    use tokio::{fs, io::AsyncReadExt};
    use zsc::THRESHOLD;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_is_spam() {
        let embeddings = embeddings::shared_embeddings().await.clone();
        let classifier = zsc::ZeroShotClassification::default(&embeddings)
            .await
            .unwrap();
        let mut entries = fs::read_dir("test_data").await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            let path = entry.path();
            if path.extension().unwrap() != "txt" {
                continue;
            }
            let mut contents = String::new();
            let mut file = fs::File::open(&path).await.unwrap();
            file.read_to_string(&mut contents).await.unwrap();

            let got = is_spam_with_custom_classifier(&embeddings, classifier.clone(), &contents)
                .await
                .unwrap();
            let expected = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("spam");

            assert_eq!(
                expected,
                got.is_spam,
                "{} was not flagged as expected",
                path.display(),
            );
            if expected {
                assert!(
                    got.score.unwrap_or(0.0) > THRESHOLD,
                    "expected score for {} to be greater than {}, got {}",
                    path.display(),
                    THRESHOLD,
                    got.score.unwrap_or(0.0)
                );
            }
        }
    }

    #[test]
    fn test_no_duplicate_test_data() {
        let dir = Path::new("test_data");
        let mut hashes: HashMap<md5::Digest, String> = HashMap::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.expect("Failed to read entry").path();
            if !path.is_file() {
                continue;
            }
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let mut contents = Vec::new();
            std::fs::File::open(path)
                .unwrap()
                .read_to_end(&mut contents)
                .unwrap();
            let hash = md5::compute(contents);
            if let Some(existing) = hashes.get(&hash) {
                panic!("Duplicate file content found: {name} and {existing}");
            } else {
                hashes.insert(hash, name);
            }
        }
    }
}
