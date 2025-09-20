use aio_translator_interface::{
    AsyncTranslator, Language, TranslationListOutput, TranslationOutput, Translator,
    TranslatorMutTrait, TranslatorTrait, error::Error, prompt::PromptBuilder,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct GoogleTranslator {
    client: Client,
    api_key: String,
}

#[derive(Serialize, Deserialize)]
struct Languages1 {
    language: String,
}
#[derive(Serialize, Deserialize)]
struct LangData {
    languages: Vec<Languages1>,
}
#[derive(Serialize, Deserialize)]
struct Langs {
    data: LangData,
}

impl Translator for GoogleTranslator {
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
impl AsyncTranslator for GoogleTranslator {
    async fn translate(
        &self,
        query: &str,
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> anyhow::Result<TranslationOutput> {
        let mut v = self
            .translate_vec(&vec![query.to_owned()], None, from, to)
            .await?;
        Ok(TranslationOutput {
            text: v.text.remove(0),
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
        let resp: Root1 = self
            .client
            .post(format!(
                "https://translation.googleapis.com/language/translate/v2?key={}",
                self.api_key
            ))
            .json(&match from {
                Some(source) => {
                    json!({"q": query, "source": source.to_google().ok_or(Error::UnknownLanguage(source))?, "target": to.to_google().ok_or(Error::UnknownLanguage(*to))?, "format": "text"})
                }
                None => json!({"q": query, "target": to.to_google().ok_or(Error::UnknownLanguage(*to))?, "format": "text"}),
            })
            .send()
            .await?
            .json()
            .await?;
        Ok(TranslationListOutput {
            text: resp
                .data
                .translations
                .into_iter()
                .map(|v| v.translated_text)
                .collect(),
            lang: None,
        })
    }
}

impl GoogleTranslator {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn languages(&self) -> Result<Vec<String>, reqwest::Error> {
        let langs: Langs = self
            .client
            .get(format!(
                "https://translation.googleapis.com/language/translate/v2/languages?key={}",
                self.api_key
            ))
            .send()
            .await?
            .json()
            .await?;
        Ok(langs
            .data
            .languages
            .into_iter()
            .map(|v| v.language)
            .collect())
    }
}

#[derive(Deserialize)]
struct Translations1 {
    #[serde(rename = "translatedText")]
    translated_text: String,
}
#[derive(Deserialize)]

struct Data1 {
    translations: Vec<Translations1>,
}
#[derive(Deserialize)]
struct Root1 {
    data: Data1,
}

#[cfg(test)]
mod tests {
    use aio_translator_interface::{Language, Translator as _};

    use crate::GoogleTranslator;

    #[tokio::test]
    async fn all_langauges_available() {
        dotenv::dotenv().ok();
        let auth = std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY not set");
        let trans = GoogleTranslator::new(auth);

        let langs = trans.languages().await.expect("Failed to fetch languages");
        assert!(langs.len() > 0);
        for lang in langs {
            if lang == "iw" {
                continue;
            }
            if lang == "jw" {
                continue;
            }
            if lang == "zh" {
                continue;
            }
            Language::from_google(&lang).expect(&lang);
        }
    }

    #[tokio::test]
    async fn translate_unknown() {
        dotenv::dotenv().ok();
        let auth = std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY not set");
        let trans = GoogleTranslator::new(auth);
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
        let auth = std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY not set");
        let trans = GoogleTranslator::new(auth);
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
