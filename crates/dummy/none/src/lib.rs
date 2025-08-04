use interface::{
    BlockingTranslator, Language, Translator, TranslatorTrait, error::Error, prompt::PromptBuilder,
};

pub struct NoneTranslator {}

impl NoneTranslator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Translator for NoneTranslator {
    fn local(&self) -> bool {
        false
    }

    fn translator<'a>(&'a self) -> interface::TranslatorTrait<'a> {
        TranslatorTrait::Blocking(self)
    }

    fn translator_mut<'a>(&'a mut self) -> interface::TranslatorMutTrait<'a> {
        interface::TranslatorMutTrait::Blocking(self)
    }
}

impl BlockingTranslator for NoneTranslator {
    fn translate(
        &mut self,
        _: &str,
        _: Option<PromptBuilder>,
        _: Language,
        _: &Language,
    ) -> Result<String, Error> {
        Ok(String::new())
    }

    fn translate_vec(
        &mut self,
        _: &[String],
        _: Option<PromptBuilder>,
        _: Language,
        _: &Language,
    ) -> Result<Vec<String>, Error> {
        Ok(Vec::new())
    }
}
