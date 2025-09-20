use std::sync::{Arc, Mutex};

use aio_translator_interface::{
    BlockingTranslator, Language, Model, Translator, TranslatorMutTrait, TranslatorTrait,
    error::Error, prompt::PromptBuilder, tokenizer::SentenceTokenizer,
};
use ct2rs::{BatchType, ComputeType, Config, Device, Tokenizer, TranslationOptions};

use interface_model::{ModelLoad, ModelSource, impl_model_load_helpers};
use maplit::hashmap;

pub struct MyTokenizer {
    tokenizer: SentenceTokenizer,
    from: Arc<Mutex<String>>,
}

impl MyTokenizer {
    pub fn new(tokenizer: SentenceTokenizer, from: Arc<Mutex<String>>) -> Self {
        Self { tokenizer, from }
    }
}

impl Tokenizer for MyTokenizer {
    fn encode(&self, input: &str) -> anyhow::Result<Vec<String>> {
        let mut encoded = self.tokenizer.encode(input)?;
        encoded.insert(0, self.from.lock().unwrap().clone());
        Ok(encoded)
    }

    fn decode(&self, tokens: Vec<String>) -> anyhow::Result<String> {
        self.tokenizer.decode(tokens)
    }
}

pub struct MBart50Translator {
    loaded_models: Option<ct2rs::Translator<MyTokenizer>>,
    cuda: bool,
    compute_type: ComputeType,
    from: Arc<Mutex<String>>,
}

impl MBart50Translator {
    /// single_loaded will only allow one model to be loaded at a time.
    pub fn new(cuda: bool, compute_type: ComputeType) -> Self {
        MBart50Translator {
            compute_type,
            cuda,
            loaded_models: None,
            from: Arc::default(),
        }
    }
}

impl Translator for MBart50Translator {
    fn local(&self) -> bool {
        true
    }
    fn translator<'a>(&'a self) -> TranslatorTrait<'a> {
        TranslatorTrait::Blocking(self)
    }

    fn translator_mut<'a>(&'a mut self) -> TranslatorMutTrait<'a> {
        TranslatorMutTrait::Blocking(self)
    }
}

impl BlockingTranslator for MBart50Translator {
    fn translate(
        &mut self,
        query: &str,
        _: Option<PromptBuilder>,
        from: Language,
        to: &Language,
    ) -> anyhow::Result<String> {
        let mut arr = self.translate_vec(&vec![query.to_owned()], None, from, to)?;
        Ok(arr.remove(0))
    }

    fn translate_vec(
        &mut self,
        query: &[String],
        _: Option<PromptBuilder>,
        from: Language,
        to: &Language,
    ) -> anyhow::Result<Vec<String>> {
        let from = from.to_mbart_50().ok_or(Error::UnknownLanguage(from))?;
        let to = to.to_mbart_50().ok_or(Error::UnknownLanguage(to.clone()))?;
        *self.from.lock().unwrap() = from.to_owned();
        let model = self.load()?;
        let trans = model.translate_batch_with_target_prefix(
            query,
            &vec![vec![to.to_string()]; query.len()],
            &TranslationOptions {
                batch_type: BatchType::Examples,
                repetition_penalty: 3.0,
                replace_unknowns: true,
                disable_unk: true,
                return_alternatives: false,
                beam_size: 5,
                ..Default::default()
            },
            None,
        )?;
        Ok(trans.into_iter().map(|v| v.0).collect())
    }
}

impl ModelLoad for MBart50Translator {
    type T = ct2rs::Translator<MyTokenizer>;

    fn loaded(&self) -> bool {
        self.loaded_models.is_some()
    }

    fn get_model(&mut self) -> Option<&mut Self::T> {
        self.loaded_models.as_mut()
    }

    fn reload(&mut self) -> anyhow::Result<&mut Self::T> {
        let model =
            self.download_model("large-many-to-many-mmt", "large-many-to-many-mmt/model.bin")?;
        let path = self.download_model("spm", "sentencepiece.bpe.model")?;
        let tokenizer = MyTokenizer::new(SentenceTokenizer::new(path), self.from.clone());
        let model = model.parent().map(|v| v.to_path_buf()).unwrap_or(model);
        let v = ct2rs::Translator::with_tokenizer(
            model,
            tokenizer,
            &Config {
                device: match self.cuda {
                    true => Device::CUDA,
                    false => Device::CPU,
                },
                compute_type: self.compute_type,
                ..Default::default()
            },
        )?;

        self.loaded_models = Some(v);
        Ok(self.loaded_models.as_mut().unwrap())
    }
}

impl Model for MBart50Translator {
    impl_model_load_helpers!("translator", "mbart50");

    fn models(&self) -> std::collections::HashMap<&'static str, interface_model::ModelSource> {
        hashmap! {
            "large-many-to-many-mmt" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/mbart-large-50-many-to-many-mmt/large-many-to-many-mmt.tar.gz",
                hash: "325b0280b362b45e4a24c68fa824549cdf9febacd4b17234cd2a57f4ec56e474",
            },

            "spm" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/mbart-large-50-many-to-many-mmt/sentencepiece.bpe.model",
                hash: "cfc8146abe2a0488e9e2a0c56de7952f7c11ab059eca145a0a727afce0db2865"
            }
        }
    }

    fn unload(&mut self) {
        self.loaded_models = None;
    }
}

#[cfg(test)]
mod tests {
    use aio_translator_interface::Language;
    use env_logger::Env;

    use super::*;

    #[test]
    fn convert_langs() {
        let langs = [
            "ar_AR", "cs_CZ", "de_DE", "en_XX", "es_XX", "et_EE", "fi_FI", "fr_XX", "gu_IN",
            "hi_IN", "it_IT", "ja_XX", "kk_KZ", "ko_KR", "lt_LT", "lv_LV", "my_MM", "ne_NP",
            "nl_XX", "ro_RO", "ru_RU", "si_LK", "tr_TR", "vi_VN", "zh_CN", "af_ZA", "az_AZ",
            "bn_IN", "fa_IR", "he_IL", "hr_HR", "id_ID", "ka_GE", "km_KH", "mk_MK", "ml_IN",
            "mn_MN", "mr_IN", "pl_PL", "ps_AF", "pt_XX", "sv_SE", "sw_KE", "ta_IN", "te_IN",
            "th_TH", "tl_XX", "uk_UA", "ur_PK", "xh_ZA", "gl_ES", "sl_SI",
        ];
        for lang in langs {
            Language::from_mbart_50(lang).expect(lang);
        }
    }
    #[test]
    fn test_load() {
        let mut nllb = MBart50Translator::new(false, ComputeType::DEFAULT);
        assert!(nllb.load().is_ok());
        assert!(nllb.loaded());
    }

    #[test]
    fn test_translate() {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
        let mut nllb = MBart50Translator::new(false, ComputeType::DEFAULT);
        let input_ja = vec![
            "明日は雨が降るかもしれません。".to_string(),
            "彼はその問題について深く考えている。".to_string(),
            "このソフトウェアは非常に使いやすいです。".to_string(),
        ];

        let out = nllb
            .translate_vec(&input_ja, None, Language::Japanese, &Language::English)
            .expect("Translation failed");
        assert_eq!(
            out,
            vec![
                "It may rain tomorrow.".to_owned(),
                "He thinks deeply about the problem.".to_owned(),
                "This software is very easy to use.".to_owned()
            ]
        );

        let input_en = vec![
            "The meeting has been postponed until next week.".to_string(),
            "She quickly realized that something was wrong.".to_string(),
            "Artificial intelligence is changing the world rapidly.".to_string(),
        ];
        let out = nllb
            .translate_vec(&input_en, None, Language::English, &Language::Japanese)
            .expect("Translation failed");
        assert_eq!(
            out,
            vec![
                "会議は来週まで延期された。".to_owned(),
                "彼女はすぐに何かが間違っていることを気付いた。".to_owned(),
                "人工知能は急速に世界を変化させています。".to_owned()
            ]
        );
    }
}
