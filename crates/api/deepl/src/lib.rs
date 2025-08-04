use std::collections::HashMap;

use aio_translator_interface::{
    AsyncTranslator, Language, TranslationListOutput, TranslationOutput, Translator,
    TranslatorMutTrait, TranslatorTrait, error::Error, prompt::PromptBuilder,
};

use anyhow::bail;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

fn most_common_string(strings: &[String]) -> Option<String> {
    let mut counts = HashMap::new();

    for s in strings {
        *counts.entry(s).or_insert(0) += 1;
    }

    counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(s, _)| s)
        .cloned()
}

pub struct DeeplTranslator {
    client: Client,
    auth: String,
}

impl DeeplTranslator {
    pub fn new(auth: String) -> Self {
        Self {
            client: Default::default(),
            auth,
        }
    }
}

impl Translator for DeeplTranslator {
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
impl AsyncTranslator for DeeplTranslator {
    async fn translate(
        &self,
        query: &str,
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        let mut t = self
            .translate_vec(&vec![query.to_owned()], None, from, to)
            .await?;
        Ok(TranslationOutput {
            text: t.text.remove(0),
            lang: t.lang,
        })
    }

    async fn translate_vec(
        &self,
        query: &[String],
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationListOutput, Error> {
        let body = match from {
            Some(s) => json!({"text": query,
                "source_lang": s.to_deepl(),
                "target_lang": to.to_deepl()
            }),
            None => json!({"text": query,
                "target_lang": to.to_deepl()}),
        };
        let request: Root1 = self
            .client
            .post("https://api-free.deepl.com/v2/translate")
            .header("Authorization", format!("DeepL-Auth-Key {}", self.auth))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        let (texts, langs): (Vec<String>, Vec<String>) = request
            .translations
            .into_iter()
            .map(|v| (v.text, v.detected_source_language))
            .unzip();
        let lang = most_common_string(&langs).ok_or(Error::CouldNotMapLanguage(None))?;
        let lang = Language::from_deepl(&lang).ok_or(Error::CouldNotMapLanguage(Some(lang)))?;
        Ok(TranslationListOutput {
            text: texts,
            lang: Some(lang),
        })
    }
}

pub async fn get_languages(auth: &String) -> anyhow::Result<Vec<String>> {
    let client = Client::new();
    let response = client
        .get("https://api-free.deepl.com/v2/languages?type=source")
        .header("Authorization", format!("DeepL-Auth-Key {}", auth))
        .header("accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        bail!(format!(
            "Request failed with status code {}",
            response.status()
        ));
    }
    let json: Vec<DeeplLanguage> = response.json().await?;
    Ok(json.iter().map(|v| v.code.to_string()).collect())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Deserialize)]
struct DeeplLanguage {
    /// identifier of language
    #[serde(alias = "language")]
    pub code: String,
    /// name of langauge
    pub name: String,
}

#[derive(Serialize, Deserialize)]
struct Translations1 {
    detected_source_language: String,
    text: String,
}
#[derive(Serialize, Deserialize)]
struct Root1 {
    translations: Vec<Translations1>,
}

#[cfg(test)]
mod tests {
    use aio_translator_interface::{Language, Translator as _};

    use crate::{DeeplTranslator, get_languages};

    #[tokio::test]
    async fn all_langauges_available() {
        dotenv::dotenv().ok();
        let auth = std::env::var("DEEPL_API_KEY").expect("DEEPL_API_KEY not set");
        let langs = get_languages(&auth)
            .await
            .expect("Failed to fetch languages");
        assert!(langs.len() > 0);
        for lang in langs {
            Language::from_deepl(&lang).expect(&lang);
        }
    }

    #[tokio::test]
    async fn translate_unknown() {
        dotenv::dotenv().ok();
        let auth = std::env::var("DEEPL_API_KEY").expect("DEEPL_API_KEY not set");
        let trans = DeeplTranslator::new(auth);
        let trans = trans.translator();
        let trans = trans.as_async().expect("Failed to create async translator");
        let trans = trans
            .translate("Hello World", None, None, &Language::German)
            .await
            .expect("Failed to translate");

        assert_eq!(trans.lang, Some(Language::English));
        assert_eq!(trans.text, "Hallo Welt");
    }

    #[tokio::test]
    async fn translate_known() {
        dotenv::dotenv().ok();
        let auth = std::env::var("DEEPL_API_KEY").expect("DEEPL_API_KEY not set");
        let trans = DeeplTranslator::new(auth);
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
