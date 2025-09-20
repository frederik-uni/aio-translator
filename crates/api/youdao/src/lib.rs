use std::time::{SystemTime, UNIX_EPOCH};

use aio_translator_interface::{
    AsyncTranslator, Language, TranslationListOutput, TranslationOutput, Translator,
    TranslatorMutTrait, TranslatorTrait, error::Error, prompt::PromptBuilder,
};
use rand::Rng as _;
use reqwest::{Client, header::CONTENT_TYPE};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::{Context, Timestamp, Uuid};

pub struct YoudaoTranslator {
    client: reqwest::Client,
    app_key: String,
    app_secret: String,
    context: Context,
    mac: [u8; 6],
}

fn generate_random_mac() -> [u8; 6] {
    let mut rng = rand::thread_rng();
    let mut mac = [0u8; 6];
    rng.fill(&mut mac);

    mac[0] |= 0b00000010;
    mac[0] &= 0b11111110;

    mac
}

impl YoudaoTranslator {
    pub fn new(app_key: String, app_secret: String) -> Self {
        let seed: u16 = rand::thread_rng().random();
        Self {
            mac: generate_random_mac(),
            client: Client::new(),
            app_key,
            app_secret,
            context: Context::new(seed),
        }
    }
}

impl Translator for YoudaoTranslator {
    fn local(&self) -> bool {
        false
    }

    fn translator<'a>(&'a self) -> TranslatorTrait<'a> {
        TranslatorTrait::Async(self)
    }

    fn translator_mut<'a>(&'a mut self) -> TranslatorMutTrait<'a> {
        TranslatorMutTrait::Async(self)
    }
}

fn sha256_encode(sign_str: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sign_str.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[async_trait::async_trait]
impl AsyncTranslator for YoudaoTranslator {
    async fn translate(
        &self,
        query: &str,
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> anyhow::Result<TranslationOutput> {
        let mut t = self
            .translate_vec(&vec![query.to_owned()], None, from, to)
            .await?;
        Ok(TranslationOutput {
            text: t.text.remove(0),
            lang: None,
        })
    }

    async fn translate_vec(
        &self,
        query: &[String],
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> anyhow::Result<TranslationListOutput> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let curtime = now.as_secs();
        let nanos = now.subsec_nanos();
        let ts = Timestamp::from_unix(&self.context, curtime, nanos);
        let salt = Uuid::new_v1(ts, &self.mac).to_string();
        let query = query.join("\n");
        let sign_str = format!(
            "{}{}{}{}{}",
            self.app_key,
            truncate(&query),
            salt,
            curtime,
            self.app_secret
        );
        let from = match from {
            Some(from) => from.to_youdao().ok_or(Error::UnknownLanguage(from))?,
            None => "auto",
        };
        let data: Resp = self
            .client
            .post("https://openapi.youdao.com/api")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&vec![
                ("from", from),
                ("to", to.to_youdao().ok_or(Error::UnknownLanguage(*to))?),
                ("signType", "v3"),
                ("curtime", &curtime.to_string()),
                ("appKey", self.app_key.as_str()),
                ("q", query.as_str()),
                ("salt", salt.as_str()),
                ("sign", &sha256_encode(&sign_str)),
            ])
            .send()
            .await?
            .json()
            .await?;
        Ok(TranslationListOutput {
            text: data
                .translation
                .into_iter()
                .flat_map(|v| v.split("/n").map(|v| v.to_owned()).collect::<Vec<String>>())
                .collect::<Vec<String>>(),
            lang: None,
        })
    }
}

#[derive(Deserialize)]
pub struct Resp {
    translation: Vec<String>,
}

fn truncate(s: &str) -> String {
    let size = s.len();
    if size <= 20 {
        s.to_string()
    } else {
        let start = &s[..10];
        let end = &s[size - 10..];
        format!("{}{}{}", start, size, end)
    }
}

#[cfg(test)]
mod tests {
    use aio_translator_interface::{Language, Translator as _};

    use crate::YoudaoTranslator;

    #[tokio::test]
    async fn all_langauges_available() {
        let langs = [
            "ar", "de", "en", "es", "fr", "hi", "id", "it", "ja", "ko", "nl", "pt", "ru", "th",
            "vi", "zh-CHS", "zh-CHT", "af", "am", "az", "be", "bg", "bn", "bs", "ca", "ceb", "co",
            "cs", "cy", "da", "el", "eo", "et", "eu", "fa", "fi", "fj", "fy", "ga", "gd", "gl",
            "gu", "ha", "haw", "he", "hi", "hr", "ht", "hu", "hy", "ig", "is", "jw", "ka", "kk",
            "km", "kn", "ku", "ky", "la", "lb", "lo", "lt", "lv", "mg", "mi", "mk", "ml", "mn",
            "mr", "ms", "mt", "mww", "my", "ne", "nl", "no", "ny", "otq", "pa", "pl", "ps", "ro",
            "sd", "si", "sk", "sl", "sm", "sn", "so", "sq", "sr-Cyrl", "sr-Latn", "st", "su", "sv",
            "sw", "ta", "te", "tg", "tl", "tlh", "to", "tr", "ty", "uk", "ur", "uz", "xh", "yi",
            "yo", "yua", "yue", "zu",
        ];

        assert!(langs.len() > 0);
        for code in langs {
            Language::from_youdao(code).expect(code);
        }
    }

    #[tokio::test]
    async fn translate_unknown() {
        dotenv::dotenv().ok();
        let key = std::env::var("YOUDAO_APP_KEY").expect("YOUDAO_APP_KEY not set");
        let secret = std::env::var("YOUDAO_APP_SECRET").expect("YOUDAO_APP_SECRET not set");
        let trans = YoudaoTranslator::new(key, secret);
        let trans = trans.translator();
        let trans = trans.as_async().expect("Failed to create async translator");
        let trans = trans
            .translate("Hello World", None, None, &Language::German)
            .await
            .expect("Failed to translate");

        assert_eq!(trans.lang, None);
        assert_eq!(trans.text, "Hallo Welt");
    }

    #[tokio::test]
    async fn translate_known() {
        dotenv::dotenv().ok();
        let key = std::env::var("YOUDAO_APP_KEY").expect("YOUDAO_APP_KEY not set");
        let secret = std::env::var("YOUDAO_APP_SECRET").expect("YOUDAO_APP_SECRET not set");
        let trans = YoudaoTranslator::new(key, secret);
        let trans = trans.translator();
        let trans = trans.as_async().expect("Failed to create async translator");
        let trans = trans
            .translate(
                "Hello World",
                None,
                Some(Language::English),
                &Language::German,
            )
            .await
            .expect("Failed to translate");

        assert_eq!(trans.lang, None);
        assert_eq!(trans.text, "Hallo Welt");
    }
}
