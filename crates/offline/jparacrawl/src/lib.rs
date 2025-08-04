use std::{collections::HashMap, path::PathBuf};

use aio_translator_interface::{
    BlockingTranslator, Language, Model, Translator, TranslatorMutTrait, TranslatorTrait,
    error::{self, Error},
    prompt::PromptBuilder,
    tokenizer::SentenceTokenizer,
};
use ct2rs::{BatchType, ComputeType, Config, Device, Tokenizer, TranslationOptions};

use interface_model::{ModelLoad, ModelLoadError, ModelSource, impl_model_load_helpers};
use maplit::hashmap;

pub struct JParaCrawlTranslator {
    single_loaded: bool,
    loaded_models: HashMap<String, ct2rs::Translator<MyTokenizer>>,
    cuda: bool,
    compute_type: ComputeType,
    size: Size,
}

pub enum Size {
    Small,
    Base,
    Large,
}

pub struct MyTokenizer {
    tokenizer_en: SentenceTokenizer,
    tokenizer_ja: SentenceTokenizer,
    en_ja: bool,
}

impl MyTokenizer {
    fn new(en_ja: bool, ja_path: PathBuf, en_path: PathBuf) -> Result<Self, ModelLoadError> {
        let tokenizer_ja = SentenceTokenizer::new(ja_path);
        let tokenizer_en = SentenceTokenizer::new(en_path);
        Ok(MyTokenizer {
            tokenizer_en,
            tokenizer_ja,
            en_ja,
        })
    }
}

impl Tokenizer for MyTokenizer {
    fn encode(&self, input: &str) -> anyhow::Result<Vec<String>> {
        match self.en_ja {
            true => &self.tokenizer_en,
            false => &self.tokenizer_ja,
        }
        .encode(input)
    }

    fn decode(&self, tokens: Vec<String>) -> anyhow::Result<String> {
        match self.en_ja {
            false => &self.tokenizer_en,
            true => &self.tokenizer_ja,
        }
        .decode(tokens)
    }
}
impl JParaCrawlTranslator {
    /// single_loaded will only allow one model to be loaded at a time.
    pub fn new(single_loaded: bool, cuda: bool, compute_type: ComputeType, size: Size) -> Self {
        JParaCrawlTranslator {
            compute_type,
            cuda,
            single_loaded,
            size,
            loaded_models: Default::default(),
        }
    }
}

impl Translator for JParaCrawlTranslator {
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

impl BlockingTranslator for JParaCrawlTranslator {
    fn translate(
        &mut self,
        query: &str,
        _: Option<PromptBuilder>,
        from: Language,
        to: &Language,
    ) -> Result<String, error::Error> {
        let mut arr = self.translate_vec(&vec![query.to_owned()], None, from, to)?;
        Ok(arr.remove(0))
    }

    fn translate_vec(
        &mut self,
        query: &[String],
        _: Option<PromptBuilder>,
        from: Language,
        to: &Language,
    ) -> Result<Vec<String>, error::Error> {
        let eng_src = match (from, to) {
            (Language::English, Language::Japanese) => true,
            (Language::Japanese, Language::English) => false,
            _ => {
                return Err(error::Error::UnknownLanguageGroup(from, to.clone()));
            }
        };
        self.load()?;

        let (from, to) = match eng_src {
            true => ("en", "ja"),
            false => ("ja", "en"),
        };
        let model_name = format!(
            "{}-{}-{}",
            from,
            to,
            match self.size {
                Size::Small => "small",
                Size::Base => "base",
                Size::Large => "big",
            }
        );
        self.custom_load(&model_name, eng_src)?;
        let trans = self
            .loaded_models
            .get_mut(&model_name)
            .expect("loaded in function")
            .translate_batch(
                query,
                &TranslationOptions {
                    batch_type: BatchType::Examples,
                    beam_size: 5,
                    repetition_penalty: 3.0,
                    num_hypotheses: 1,
                    replace_unknowns: true,
                    disable_unk: true,
                    return_alternatives: false,
                    ..Default::default()
                },
                None,
            )
            .map_err(Error::CTranslator)?;

        Ok(trans.into_iter().map(|v| v.0).collect())
    }
}

impl JParaCrawlTranslator {
    fn custom_load(&mut self, name: &str, en_ja: bool) -> Result<(), error::Error> {
        if self.loaded_models.contains_key(name) {
            return Ok(());
        }
        let model = self.download_model(name, &format!("{}/model.bin", name))?;
        let ja_path = self.download_model("spm.nopretok", "spm.nopretok/spm.ja.nopretok.model")?;
        let en_path = self.download_model("spm.nopretok", "spm.nopretok/spm.en.nopretok.model")?;

        let model = model.parent().map(|v| v.to_path_buf()).unwrap_or(model);
        let my = MyTokenizer::new(en_ja, ja_path, en_path)?;

        let v = ct2rs::Translator::with_tokenizer(
            model,
            my,
            &Config {
                device: match self.cuda {
                    true => Device::CUDA,
                    false => Device::CPU,
                },
                compute_type: self.compute_type,
                ..Default::default()
            },
        )
        .map_err(Error::CTranslator)?;
        if self.single_loaded {
            self.loaded_models.drain();
        }
        self.loaded_models.insert(name.to_owned(), v);
        Ok(())
    }
}

impl Model for JParaCrawlTranslator {
    impl_model_load_helpers!("translator", "JParaCrawl");

    fn models(&self) -> std::collections::HashMap<&'static str, interface_model::ModelSource> {
        hashmap! {
            "ja-en-big" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/jparacrawl-3.0/ja-en-big.tar.gz",
                hash: "188191b34a2002ebc9fba6c8b6e7e803006d65abe583769f20bd50a934a0be33",
            },
            "ja-en-base" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/jparacrawl-3.0/ja-en-base.tar.gz",
                hash: "a3d63137128c08283299738d5dba6c13930b3b0f651a7265d332aa2032ef5e4d",
            },
            "ja-en-small" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/jparacrawl-3.0/ja-en-small.tar.gz",
                hash: "c29c75b5637d1b6836f353b31cbd9af60c061e53d73152d20763ce30daa79ff0",
            },
            "en-ja-big" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/jparacrawl-3.0/en-ja-big.tar.gz",
                hash: "aaf9bb8a42f128d0f31cc97809f4b259b90a96e1afc507ca807c773d02123544",
            },
            "en-ja-base" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/jparacrawl-3.0/en-ja-base.tar.gz",
                hash: "1a3f9b5cf1220af7b955223d0e29987ff71e8526808103f9909227d936c00ca7",
            },
            "en-ja-small" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/jparacrawl-3.0/en-ja-small.tar.gz",
                hash: "3726e02ee99190c25b2ab976e673d85dcc6dc62183527baa1ce84f9ff630eeec"
            },
            "spm.nopretok" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/jparacrawl-3.0/spm.nopretok.tar.gz",
                hash: "ba95a8e1767df22e8d7aecbba76c418a9225ec38705955a736509979f7f5c770"
            }
        }
    }

    fn unload(&mut self) {
        self.loaded_models = HashMap::new();
    }
}

impl ModelLoad for JParaCrawlTranslator {
    type T = HashMap<String, ct2rs::Translator<MyTokenizer>>;

    fn loaded(&self) -> bool {
        self.loaded_models.len() > 0
    }

    fn get_model(&mut self) -> Option<&mut Self::T> {
        Some(&mut self.loaded_models)
    }

    fn reload(&mut self) -> Result<&mut Self::T, ModelLoadError> {
        Ok(&mut self.loaded_models)
    }
}

#[cfg(test)]
mod tests {
    use env_logger::Env;

    use super::*;

    #[test]
    fn test_load() {
        let mut jparacrawl = JParaCrawlTranslator::new(false, false, ComputeType::INT8, Size::Base);
        assert!(jparacrawl.load().is_ok());
    }

    #[test]
    fn test_translate() {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
        let mut jparacrawl =
            JParaCrawlTranslator::new(false, false, ComputeType::DEFAULT, Size::Base);
        let input_ja = vec![
            "明日は雨が降るかもしれません。".to_string(),
            "彼はその問題について深く考えている。".to_string(),
            "このソフトウェアは非常に使いやすいです。".to_string(),
        ];

        let out = jparacrawl
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
        let out = jparacrawl
            .translate_vec(&input_en, None, Language::English, &Language::Japanese)
            .expect("Translation failed");
        assert_eq!(
            out,
            vec![
                "会議は来週まで延期されました。".to_string(),
                "彼女はすぐに何かが間違っていることに気づきました。".to_string(),
                "人工知能は急速に世界を変えています。".to_string()
            ]
        );
    }
}
