use aio_translator_interface::{
    BlockingTranslator, Language, Translator, TranslatorMutTrait, TranslatorTrait,
    prompt::PromptBuilder,
};

pub struct OriginalTranslator {}

impl OriginalTranslator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Translator for OriginalTranslator {
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

impl BlockingTranslator for OriginalTranslator {
    fn translate(
        &mut self,
        input: &str,
        _: Option<PromptBuilder>,
        _: Language,
        _: &Language,
    ) -> anyhow::Result<String> {
        Ok(input.to_owned())
    }

    fn translate_vec(
        &mut self,
        items: &[String],
        _: Option<PromptBuilder>,
        _: Language,
        _: &Language,
    ) -> anyhow::Result<Vec<String>> {
        Ok(items.to_vec())
    }
}
