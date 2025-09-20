use crate::Language;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to fetch")]
    Reqwest(#[from] reqwest::Error),
    #[error("Api returned invalid response")]
    ApiError(ApiError),
    #[error("Couldnt convert language")]
    UnknownLanguage(Language),
    #[error("Translator doesnt support this language")]
    UnknownLanguageGroup(Language, Language),
    #[error("failed to convert string to language")]
    CouldNotMapLanguage(Option<String>),
    #[error("api did not return a response")]
    NoResponse,
    #[error("Request was too long")]
    RequestToLong(u32, u32),
    #[error("Request failed with status code")]
    RequestFailed(u16),
}

#[derive(Debug)]
pub enum ApiError {
    Baidu { code: String, message: String },
}
