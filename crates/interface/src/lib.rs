pub mod error;
pub mod prompt;
pub mod tokenizer;

use crate::{error::Error, prompt::PromptBuilder};
pub use interface_model::Model;
use lang_generator::generate_language;

generate_language!();

pub trait Translator {
    fn local(&self) -> bool;
    fn translator<'a>(&'a self) -> TranslatorTrait<'a>;
    fn translator_mut<'a>(&'a mut self) -> TranslatorMutTrait<'a>;
}

pub trait Detector {
    fn detect_language(&self, text: &str) -> Option<Language>;
}

pub trait BlockingTranslator: Send + Sync {
    fn translate(
        &mut self,
        query: &str,
        context: Option<PromptBuilder>,
        from: Language,
        to: &Language,
    ) -> Result<String, Error>;

    fn translate_vec(
        &mut self,
        query: &[String],
        context: Option<PromptBuilder>,
        from: Language,
        to: &Language,
    ) -> Result<Vec<String>, Error>;
}

pub enum TranslatorTrait<'a> {
    Async(&'a dyn AsyncTranslator),
    Blocking(&'a dyn BlockingTranslator),
}

pub enum TranslatorMutTrait<'a> {
    Async(&'a mut dyn AsyncTranslator),
    Blocking(&'a mut dyn BlockingTranslator),
}

impl TranslatorMutTrait<'_> {
    pub fn as_blocking(&mut self) -> Option<&mut dyn BlockingTranslator> {
        match self {
            TranslatorMutTrait::Async(_) => None,
            TranslatorMutTrait::Blocking(translator) => Some(*translator),
        }
    }
}

impl TranslatorTrait<'_> {
    pub fn as_async(&self) -> Option<&dyn AsyncTranslator> {
        match self {
            TranslatorTrait::Async(translator) => Some(*translator),
            TranslatorTrait::Blocking(_) => None,
        }
    }
}

#[async_trait::async_trait]
pub trait AsyncTranslator: Send + Sync {
    async fn translate(
        &self,
        query: &str,
        context: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error>;

    async fn translate_vec(
        &self,
        query: &[String],
        context: Option<PromptBuilder>,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationListOutput, Error>;
}

/// Translation Result containing the translation and the language
#[derive(Clone, Debug)]
pub struct TranslationOutput {
    /// Translation
    pub text: String,
    /// Text language
    pub lang: Option<Language>,
}

/// Translation Result containing the translation and the language
#[derive(Clone, Debug)]
pub struct TranslationListOutput {
    /// Translation
    pub text: Vec<String>,
    /// Text language
    pub lang: Option<Language>,
}
