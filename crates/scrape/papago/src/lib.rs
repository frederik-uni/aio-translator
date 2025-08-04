use std::time::{SystemTime, UNIX_EPOCH};

use aio_translator_interface::error::Error;
use aio_translator_interface::prompt::PromptBuilder;
use aio_translator_interface::{
    AsyncTranslator, Language, TranslationListOutput, TranslationOutput, Translator,
    TranslatorMutTrait, TranslatorTrait,
};
use base64::Engine;
use hmac::{Hmac, Mac};
use md5::Md5;
use regex::Regex;
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

pub struct PapagoTranslator {
    client: Client,
    ver: String,
    honorific: bool,
}

impl PapagoTranslator {
    pub async fn new(honorific: bool) -> Result<PapagoTranslator, Error> {
        let client = Client::new();
        let ver = version_key(&client).await?;

        Ok(PapagoTranslator {
            client,
            ver,
            honorific,
        })
    }
}

impl Translator for PapagoTranslator {
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

#[async_trait::async_trait]
impl AsyncTranslator for PapagoTranslator {
    async fn translate(
        &self,
        query: &str,
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        let from = from
            .map(|v| v.to_papago().ok_or(Error::UnknownLanguage(v)))
            .unwrap_or(Ok("auto"))?;
        let to = to.to_papago().ok_or(Error::UnknownLanguage(*to))?;
        let url = "https://papago.naver.com/apis/n2mt/translate";

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let ppg = get_auth_ppg(url, &self.ver, &uuid::Uuid::new_v4().to_string(), ts)?;
        let content: Root1 = self
            .client
            .post(url)
            .header(AUTHORIZATION, ppg)
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("Timestamp", ts.to_string())
            .form(&vec![
                ("honorific", self.honorific.to_string().as_str()),
                ("source", from),
                ("target", to),
                ("text", query),
            ])
            .send()
            .await?
            .json()
            .await?;
        let lang = content
            .lang_detection
            .nbests
            .into_iter()
            .max_by(|a, b| {
                a.prob
                    .partial_cmp(&b.prob)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|v| v.lang)
            .ok_or(Error::CouldNotMapLanguage(None))?;
        let lang = Language::from_papago(&lang).ok_or(Error::CouldNotMapLanguage(Some(lang)))?;
        Ok(TranslationOutput {
            text: content.translated_text,
            lang: Some(lang),
        })
    }

    async fn translate_vec(
        &self,
        query: &[String],
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationListOutput, Error> {
        let t = self.translate(&query.join("\n"), None, from, to).await?;
        Ok(TranslationListOutput {
            text: t
                .text
                .split("\n")
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            lang: t.lang,
        })
    }
}

fn get_auth_ppg(
    url: &str,
    auth_key: &str,
    device_id: &str,
    time_stamp: u64,
) -> Result<String, Error> {
    let value = format!(
        "{}\n{}\n{}",
        device_id,
        url.split('?').next().unwrap_or(url),
        time_stamp
    );
    let mut mac = Hmac::<Md5>::new_from_slice(auth_key.as_bytes()).unwrap();
    mac.update(value.as_bytes());
    let result = mac.finalize().into_bytes();
    Ok(format!(
        "PPG {}:{}",
        device_id,
        base64::engine::general_purpose::STANDARD.encode(result)
    ))
}
async fn version_key(client: &Client) -> Result<String, Error> {
    let script = client
        .get("https://papago.naver.com")
        .send()
        .await?
        .text()
        .await?;
    let main_js = Regex::new(r"\/(main.*\.js)")
        .expect("Regex")
        .captures(&script)
        .unwrap()[1]
        .to_string();
    let ver_data = client
        .get(format!("https://papago.naver.com/{}", main_js))
        .send()
        .await?
        .text()
        .await?;
    Ok(Regex::new(r#""PPG .*,"(v[^"]*)"#)
        .expect("Regex")
        .captures(&ver_data)
        .unwrap()[1]
        .to_string())
}

async fn get_languages() -> Result<Vec<String>, Error> {
    let host = "https://papago.naver.com";
    let client = Client::new();

    let data = client.get(host).send().await?.text().await?;
    let url_path = Regex::new(r"/home\.(.*?)\.chunk\.js")
        .expect("Regex")
        .captures(&data)
        .unwrap()[0]
        .to_string();
    let lang_detect_url = format!("{}{}", host, url_path);

    let lang_html = client.get(lang_detect_url).send().await?.text().await?;
    let lang_re = Regex::new(r#"=\{ALL:(.*?)}"#).expect("Regex");
    let lang_str = lang_re.captures(&lang_html).unwrap()[0]
        .to_owned()
        .to_lowercase()
        .replace("zh-cn", "zh-CN")
        .replace("zh-tw", "zh-TW")
        .replace('\"', "");
    let lang_re2 = Regex::new(r#","(.*?)":|,(.*?):"#).expect("Regex");
    let lang_list: Vec<String> = lang_re2
        .find_iter(&lang_str)
        .map(|m| m.as_str().trim_matches(|c| c == ',' || c == ':').to_owned())
        .filter(|x| x != "auto")
        .collect();

    Ok(lang_list)
}

#[derive(Serialize, Deserialize)]
struct Nbests1 {
    lang: String,
    prob: f64,
}
#[derive(Serialize, Deserialize)]
struct LangDetection1 {
    nbests: Vec<Nbests1>,
}

#[derive(Serialize, Deserialize)]
struct Root1 {
    #[serde(rename = "langDetection")]
    lang_detection: LangDetection1,
    #[serde(rename = "translatedText")]
    translated_text: String,
}

#[cfg(test)]
mod tests {
    use aio_translator_interface::{Language, Translator as _};

    use crate::{PapagoTranslator, get_languages};

    #[tokio::test]
    async fn all_langauges_available() {
        let langs = get_languages().await.expect("Failed to fetch languages");
        assert!(langs.len() > 0);
        for lang in langs {
            Language::from_papago(&lang).expect(&lang);
        }
    }

    #[tokio::test]
    async fn translate_unknown() {
        let trans = PapagoTranslator::new(false)
            .await
            .expect("Failed to create translator");
        let trans = trans.translator();
        let trans = trans.as_async().expect("Failed to create async translator");
        let trans = trans
            .translate_vec(
                &vec!["Hello World".to_owned(), "This is a test".to_owned()],
                None,
                None,
                &Language::German,
            )
            .await
            .expect("Failed to translate");

        assert_eq!(trans.lang, Some(Language::English));
        assert_eq!(trans.text[0], "Hallo Welt");
        assert_eq!(trans.text[1], "Das ist ein Test.");
    }

    #[tokio::test]
    async fn translate_known() {
        let trans = PapagoTranslator::new(false)
            .await
            .expect("Failed to create translator");
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

        assert_eq!(trans.lang, Some(Language::English));
        assert_eq!(trans.text, "Hallo Welt");
    }
}
