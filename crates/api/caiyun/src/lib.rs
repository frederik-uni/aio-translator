use interface::{
    AsyncTranslator, Language, TranslationListOutput, TranslationOutput, Translator,
    TranslatorTrait, error::Error, prompt::PromptBuilder,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct CaiyunRequest<'a> {
    trans_type: String,
    source: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    detect: Option<bool>,
    request_id: &'a str,
}

#[derive(Deserialize)]
struct CaiyunResponse {
    target: Option<Vec<String>>,
}

pub struct CaiyunTranslator {
    client: Client,
    token: String,
    request_id: String,
}

#[async_trait::async_trait]
impl AsyncTranslator for CaiyunTranslator {
    async fn translate(
        &self,
        query: &str,
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
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
    ) -> Result<TranslationListOutput, Error> {
        let f = from;
        let from = match from {
            Some(from) => from.to_caiyun().ok_or(Error::UnknownLanguage(from))?,
            None => "auto",
        };
        let trans_type = format!(
            "{}2{}",
            from,
            to.to_caiyun().ok_or(Error::UnknownLanguage(*to))?
        );

        let request = CaiyunRequest {
            trans_type,
            source: query,
            detect: if f.is_none() { Some(true) } else { None },
            request_id: &self.request_id,
        };
        let data: CaiyunResponse = self
            .client
            .post("https://api.interpreter.caiyunai.com/v1/translator")
            .header("content-type", "application/json")
            .header("x-authorization", format!("token {}", self.token))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        Ok(TranslationListOutput {
            text: data.target.unwrap_or_default(),
            lang: None,
        })
    }
}

impl CaiyunTranslator {
    pub fn new(token: String, request_id: String) -> Self {
        Self {
            client: Client::new(),
            token,
            request_id,
        }
    }
}

impl Translator for CaiyunTranslator {
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

#[cfg(test)]
mod tests {
    use interface::{Language, Translator as _};

    use crate::CaiyunTranslator;

    #[tokio::test]
    async fn all_langauges_available() {
        dotenv::dotenv().ok();
        //TODO: Arabic, Greek, Indonesian, Polish, Swahili, Thai,
        let langs = vec![
            "zh", "zh-Hant", "en", "ja", "ko", "de", "fr", "es", "it", "pt", "ru", "tr", "vi",
        ];

        for lang in langs {
            Language::from_caiyun(&lang).expect(&lang);
        }
    }

    #[tokio::test]
    async fn translate_unknown() {
        dotenv::dotenv().ok();
        let auth = std::env::var("CAIYUN_TOKEN").expect("CAIYUN_TOKEN not set");
        let trans = CaiyunTranslator::new(auth, "demo".to_string());
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
        let auth = std::env::var("CAIYUN_TOKEN").expect("CAIYUN_TOKEN not set");
        let trans = CaiyunTranslator::new(auth, "demo".to_string());
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
