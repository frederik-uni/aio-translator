mod style_transfer;

pub use interface::{
    AsyncTranslator, BlockingTranslator, Detector, Language, Model, TranslationListOutput,
    TranslationOutput, Translator, TranslatorTrait, error::ApiError, error::Error,
    prompt::PromptBuilder,
};

pub use baidu::BaiduTranslator;
pub use caiyun::CaiyunTranslator;
pub use ct2rs::ComputeType;
pub use deepl::DeeplTranslator;
pub use google::GoogleTranslator;
pub use jparacrawl::JParaCrawlTranslator;
pub use langid::LangIdDetector;
#[cfg(feature = "lingua")]
pub use lingua::LinguaDetector;
pub use m2m100::M2M100Translator;
pub use mymemory::MyMemoryTranslator;
pub use nllb::NLLBTranslator;
pub use none::NoneTranslator;
pub use original::OriginalTranslator;
pub use papago::PapagoTranslator;
pub use style_transfer::StyleTransfer;
pub use sugoi::SugoiTranslator;
#[cfg(feature = "whatlang")]
pub use whatlang::WhatLangDetector;
pub use youdao::YoudaoTranslator;

#[cfg(test)]
mod tests {

    #[test]
    fn test_style_transfer() {
        pub use crate::Translator;
        let cuda = true;
        let mut t = crate::SugoiTranslator::new(cuda, crate::ComputeType::DEFAULT);
        t.translator_mut()
            .as_blocking()
            .unwrap()
            .translate_vec(
                &["Hello World".to_owned()],
                None,
                crate::Language::Japanese,
                &crate::Language::English,
            )
            .unwrap();
    }
}
