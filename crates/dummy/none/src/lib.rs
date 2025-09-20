use aio_translator_interface::{
    BlockingTranslator, Language, Translator, TranslatorMutTrait, TranslatorTrait,
    prompt::PromptBuilder,
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

    fn translator<'a>(&'a self) -> TranslatorTrait<'a> {
        TranslatorTrait::Blocking(self)
    }

    fn translator_mut<'a>(&'a mut self) -> TranslatorMutTrait<'a> {
        TranslatorMutTrait::Blocking(self)
    }
}

impl BlockingTranslator for NoneTranslator {
    fn translate(
        &mut self,
        _: &str,
        _: Option<PromptBuilder>,
        _: Language,
        _: &Language,
    ) -> anyhow::Result<String> {
        Ok(String::new())
    }

    fn translate_vec(
        &mut self,
        _: &[String],
        _: Option<PromptBuilder>,
        _: Language,
        _: &Language,
    ) -> anyhow::Result<Vec<String>> {
        Ok(Vec::new())
    }
}
