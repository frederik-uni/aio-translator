//https://docs.rs/crate/translation-api-cn/latest/source/src/baidu.rs

use aio_translator_interface::{
    AsyncTranslator, Language, TranslationListOutput, TranslationOutput, Translator,
    TranslatorMutTrait, TranslatorTrait,
    error::{ApiError, Error},
    prompt::PromptBuilder,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub struct BaiduTranslator {
    url: String,
    app_id: String,
    key: String,
    client: Client,
}

impl Translator for BaiduTranslator {
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

#[async_trait]
impl AsyncTranslator for BaiduTranslator {
    async fn translate(
        &self,
        query: &str,
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        let to = to.to_baidu().ok_or(Error::UnknownLanguage(to.clone()))?;
        let from = match from {
            Some(item) => item.to_baidu().ok_or(Error::UnknownLanguage(item))?,
            None => "auto",
        };
        let form = Form::new(&self.app_id, query, "0", &self.key, from, to);
        let resp: Response = self
            .client
            .post(&self.url)
            .form(&form)
            .send()
            .await?
            .json()
            .await?;
        let resp = match resp {
            Response::Ok(v) => v,
            Response::Err(v) => {
                return Err(Error::ApiError(ApiError::Baidu {
                    message: v.solution().to_owned(),
                    code: v.code,
                }));
            }
        };
        Ok(TranslationOutput {
            text: resp
                .trans_result
                .iter()
                .map(|v| v.dst.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            lang: Some(
                Language::from_baidu(&resp.to).ok_or(Error::CouldNotMapLanguage(Some(resp.to)))?,
            ),
        })
    }

    async fn translate_vec(
        &self,
        query: &[String],
        _: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationListOutput, Error> {
        let v = self.translate(&query.join("\n"), None, from, to).await?;
        Ok(TranslationListOutput {
            text: v.text.split('\n').map(|v| v.to_string()).collect(),
            lang: v.lang,
        })
    }
}

impl BaiduTranslator {
    pub fn new(app_id: &str, key: &str) -> Self {
        Self {
            url: "https://fanyi-api.baidu.com/api/trans/vip/translate".to_string(),
            app_id: app_id.to_string(),
            key: key.to_string(),
            client: Client::new(),
        }
    }
}

/// The data submitted by the form
#[derive(Debug, Serialize)]
pub struct Form {
    pub q: String,
    pub from: String,
    pub to: String,
    pub appid: String,
    pub salt: String,
    pub sign: String,
}

impl Form {
    fn new(appid: &str, q: &str, salt: &str, key: &str, from: &str, to: &str) -> Self {
        let data = format!("{}{}{}{}", &appid, q, salt, key);
        let sign = format!("{:x}", md5::compute(data));
        Self {
            q: q.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            appid: appid.to_string(),
            salt: salt.to_string(),
            sign,
        }
    }
}

/// Response information. Either return the translation result, or return an error message.
#[derive(Deserialize)]
#[serde(untagged)]
enum Response {
    Ok(TranslationResponse),
    Err(BaiduApiError),
}

/// Error handling / error code
#[derive(Debug, Clone, Deserialize)]
pub struct BaiduApiError {
    #[serde(rename = "error_code")]
    pub code: String,
    #[serde(rename = "error_msg")]
    pub msg: String,
    pub data: Option<Value>,
}

impl std::error::Error for BaiduApiError {}
impl std::fmt::Display for BaiduApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error code: `{}`\nError message: `{}`\nError meaning: {}\nThe above content is returned by Baidu translation API",
            self.code,
            self.msg,
            self.solution()
        )
    }
}

impl BaiduApiError {
    ///Reference: [Error Code List](https://fanyi-api.baidu.com/doc/21)
    pub fn solution(&self) -> &str {
        match self.code.as_bytes() {
            b"52000" => "success",
            b"52001" => "Request timed out. \nSolution: Please try again.",
            b"52002" => "system error. \nSolution: Please try again.",
            b"52003" => {
                "Unauthorized user. \nSolution: Please check whether the appid is correct or whether the service is enabled."
            }
            b"54000" => {
                "The required parameter is empty. \nSolution: Please check whether to pass fewer parameters."
            }
            b"54001" => {
                "Wrong signature. \nSolution: Please check your signature generation method."
            }
            b"54003" => {
                "Frequency of access is limited. \nSolution: Please reduce your calling frequency, or switch to the premium version after authentication."
            }
            b"54004" => {
                "Insufficient account balance. \nSolution: Please go to the management console to recharge the account."
            }
            b"54005" => {
                "Long query requests are frequent. \nSolution: Please reduce the sending frequency of long queries and try again after 3s."
            }
            b"58000" => {
                "Client IP is illegal. \nSolution: Check whether the IP address filled in the personal information is correct, and you can go to Developer Information-Basic Information to modify it."
            }
            b"58001" => {
                "Target language direction is not supported. \nSolution: Check if the target language is in the language list."
            }
            b"58002" => {
                "The service is currently down. \nSolution: Please go to the management console to enable the service."
            }
            b"58003" => {
                "f the same IP uses multiple APPIDs to send translation requests on the same day, the IP will be banned from making requests for the rest of that day and will be unblocked the next day. Do not enter your APPID and secret key into third-party software."
            }
            b"90107" => {
                "The certification has not passed or is not valid. \nSolution: Please go to My Certification to check the certification progress."
            }
            b"20003" => {
                "Please check whether the request text involves content related to subversion, violence, or similar topics."
            }
            _ => "unknown error",
        }
    }
}

#[derive(Deserialize)]
struct Sentence {
    // pub src: String,
    pub dst: String,
}

#[derive(Deserialize)]
struct TranslationResponse {
    // pub from: String,
    pub to: String,
    pub trans_result: Vec<Sentence>,
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use aio_translator_interface::Language;

    use crate::BaiduTranslator;

    #[tokio::test]
    async fn translate() {
        //TODO:
        BaiduTranslator::new("", "");
    }

    #[test]
    fn map_langs() {
        let langs = [
            "zh", "en", "yue", "wyw", "jp", "kor", "fra", "spa", "th", "ara", "ru", "pt", "de",
            "it", "el", "nl", "pl", "bul", "est", "dan", "fin", "cs", "rom", "slo", "swe", "hu",
            "cht", "vie", "ara", "gle", "oci", "alb", "arq", "aka", "arg", "amh", "asm", "aym",
            "aze", "ast", "oss", "est", "oji", "ori", "orm", "pl", "per", "bre", "bak", "baq",
            "pot", "bel", "ber", "pam", "bul", "sme", "ped", "bem", "bli", "bis", "bal", "ice",
            "bos", "bho", "chv", "tso", "dan", "de", "tat", "sha", "tet", "div", "log", "ru",
            "fra", "fil", "fin", "san", "fri", "ful", "fao", "gla", "kon", "ups", "hkm", "kal",
            "geo", "guj", "gra", "eno", "grn", "kor", "nl", "hup", "hak", "ht", "mot", "hau",
            "kir", "glg", "frn", "cat", "cs", "kab", "kan", "kau", "kah", "cor", "xho", "cos",
            "cre", "cri", "kli", "hrv", "que", "kas", "kok", "kur", "lat", "lao", "rom", "lag",
            "lav", "lim", "lin", "lug", "ltz", "ruy", "kin", "lit", "roh", "ro", "loj", "may",
            "bur", "mar", "mg", "mal", "mac", "mah", "mai", "glv", "mau", "mao", "ben", "mlt",
            "hmn", "nor", "nea", "nbl", "afr", "sot", "nep", "pt", "pan", "pap", "pus", "nya",
            "twi", "chr", "jp", "swe", "srd", "sm", "sec", "srp", "sol", "sin", "epo", "nob", "sk",
            "slo", "swa", "src", "som", "sco", "th", "tr", "tgk", "tam", "tgl", "tir", "tel",
            "tua", "tuk", "ukr", "wln", "wel", "ven", "wol", "urd", "spa", "heb", "el", "hu",
            "fry", "sil", "hil", "los", "haw", "nno", "nqo", "snd", "sna", "ceb", "syr", "sun",
            "en", "hi", "id", "it", "vie", "yid", "ina", "ach", "ing", "ibo", "ido", "yor", "arm",
            "iku", "zh", "cht", "wyw", "yue", "zaz", "frm", "zul", "jav",
        ];
        for lang_str in langs.into_iter().collect::<HashSet<_>>() {
            if lang_str == "slo" {
                continue;
            }
            Language::from_baidu(lang_str).expect(lang_str);
        }
    }
}
