use interface::{
    AsyncTranslator, Language, TranslationListOutput, TranslationOutput, Translator,
    TranslatorTrait, error::Error, prompt::PromptBuilder,
};
use reqwest::{Client, header::REFERER};
use serde_json::Value;

pub struct MyMemoryTranslator {
    /// how long the text to translate can be
    input_limit: u32,
    /// host url
    host: String,
    client: Client,
}

/// default value
impl Default for MyMemoryTranslator {
    /// new is default
    fn default() -> Self {
        MyMemoryTranslator::new()
    }
}

pub fn input_limit_checker(query: &str, input_limit: u32) -> Result<(), Error> {
    if query.len() > input_limit as usize {
        return Err(Error::RequestToLong(query.len() as u32, input_limit));
    }
    Ok(())
}

impl Translator for MyMemoryTranslator {
    fn local(&self) -> bool {
        false
    }

    fn translator<'a>(&'a self) -> interface::TranslatorTrait<'a> {
        TranslatorTrait::Async(self)
    }

    fn translator_mut<'a>(&'a mut self) -> interface::TranslatorMutTrait<'a> {
        interface::TranslatorMutTrait::Async(self)
    }
}

#[async_trait::async_trait]
impl AsyncTranslator for MyMemoryTranslator {
    async fn translate(
        &self,
        query: &str,
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        input_limit_checker(query, self.input_limit)?;
        let from_orig = from;
        let from = match from {
            Some(lang) => lang.to_mymemory().ok_or(Error::UnknownLanguage(lang))?,
            None => "Autodetect",
        };
        let url = format!(
            "{}?q={}&langpair={}|{}",
            self.host,
            query,
            from,
            to.to_mymemory().ok_or(Error::UnknownLanguage(*to))?
        );

        let response = self
            .client
            .get(&url)
            .header(REFERER, "https://mymemory.translated.net")
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(Error::RequestFailed(response.status().as_u16()));
        }
        let resp: Value = response.json().await?;
        let resp = &resp["responseData"];
        let lang = resp["detectedLanguage"].to_string();
        let mut text = resp["translatedText"].to_string();
        if text == "null" {
            return Err(Error::NoResponse);
        }
        if text.starts_with('"') && text.ends_with('"') {
            text = text[1..text.len() - 1].to_string();
        }
        Ok(TranslationOutput {
            text,
            lang: Some(match from_orig {
                Some(s) => s,
                None => Language::from_mymemory_short(&lang.replace("\"", ""))
                    .ok_or(Error::CouldNotMapLanguage(Some(lang)))?,
            }),
        })
    }

    async fn translate_vec(
        &self,
        query: &[String],
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationListOutput, Error> {
        let t = self.translate(&query.join("_._._"), None, from, to).await?;
        Ok(TranslationListOutput {
            text: t
                .text
                .split("_._._")
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            lang: t.lang,
        })
    }
}

impl MyMemoryTranslator {
    pub fn new() -> Self {
        MyMemoryTranslator {
            client: Default::default(),
            input_limit: 500,
            host: "https://api.mymemory.translated.net/get".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use interface::{Language, Translator as _};
    use reqwest::Client;
    use scraper::{Html, Selector};

    use crate::MyMemoryTranslator;

    #[tokio::test]
    async fn translate_unknown() {
        dotenv::dotenv().ok();
        let trans = MyMemoryTranslator::new();
        let trans = trans.translator();
        let trans = trans.as_async().expect("Failed to create async translator");
        let trans = trans
            .translate("Hello World", None, None, &Language::German)
            .await
            .expect("Failed to translate");

        assert_eq!(trans.lang, Some(Language::English));
        assert_eq!(trans.text, "Hallo Welt");
    }

    pub async fn get_languages() -> Vec<String> {
        let client = Client::new();
        let data = client
            .get("https://mymemory.translated.net")
            .send()
            .await
            .expect("Failed to send request");
        if !data.status().is_success() {
            panic!("Request failed with status code {}", data.status())
        }
        let data = data.text().await.expect("Failed to get response text");
        let document = Html::parse_document(&data);
        let selector =
            Selector::parse("#select_source_mm > option").expect("Failed to parse selector");
        document
            .select(&selector)
            .map(|v| v.attr("value").expect("Failed to get value").to_owned())
            .collect()
    }

    #[tokio::test]
    async fn all_langauges_available() {
        let langs = get_languages().await;
        assert!(langs.len() > 0);
        for lang in langs {
            if lang.as_str() == "Autodetect" {
                continue;
            }
            if lang.as_str() == "------" {
                continue;
            }

            Language::from_mymemory(&lang).expect(&lang);
        }
    }

    #[tokio::test]
    async fn translate_known() {
        dotenv::dotenv().ok();
        let trans = MyMemoryTranslator::new();
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
