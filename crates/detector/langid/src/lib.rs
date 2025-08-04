use std::sync::Arc;

use aio_translator_interface::{Detector, Language};

#[derive(Clone)]
pub struct LangIdDetector {
    m: Arc<langid_rs::Model>,
}

impl LangIdDetector {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            m: Arc::new(langid_rs::Model::load(false)?),
        })
    }
}

impl Detector for LangIdDetector {
    fn detect_language(&self, s: &str) -> Option<Language> {
        let (lang, _) = self.m.classify(s)?;
        Language::from_639_1(lang)
    }
}
