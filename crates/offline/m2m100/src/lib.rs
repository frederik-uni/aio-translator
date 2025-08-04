use std::sync::{Arc, Mutex};

use ct2rs::{BatchType, ComputeType, Config, Device, Tokenizer, TranslationOptions};
use interface::{
    BlockingTranslator, Model, Translator, TranslatorTrait, error::Error, prompt::PromptBuilder,
};

use interface_model::{ModelLoad, ModelLoadError, ModelSource, impl_model_load_helpers};
use maplit::hashmap;

pub struct MyTokenizer {
    tokenizer: interface::tokenizer::SentenceTokenizer,
    from: Arc<Mutex<String>>,
}

impl MyTokenizer {
    pub fn new(
        tokenizer: interface::tokenizer::SentenceTokenizer,
        from: Arc<Mutex<String>>,
    ) -> Self {
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

pub struct M2M100Translator {
    loaded_models: Option<ct2rs::Translator<MyTokenizer>>,
    cuda: bool,
    compute_type: ComputeType,
    from: Arc<Mutex<String>>,
    size: Size,
}

pub enum Size {
    Small,
    Large,
}

impl M2M100Translator {
    /// single_loaded will only allow one model to be loaded at a time.
    pub fn new(cuda: bool, compute_type: ComputeType, size: Size) -> Self {
        M2M100Translator {
            compute_type,
            cuda,
            size,
            loaded_models: None,
            from: Arc::default(),
        }
    }
}

impl Translator for M2M100Translator {
    fn local(&self) -> bool {
        true
    }

    fn translator<'a>(&'a self) -> interface::TranslatorTrait<'a> {
        TranslatorTrait::Blocking(self)
    }

    fn translator_mut<'a>(&'a mut self) -> interface::TranslatorMutTrait<'a> {
        interface::TranslatorMutTrait::Blocking(self)
    }
}

impl BlockingTranslator for M2M100Translator {
    fn translate(
        &mut self,
        query: &str,
        _: Option<PromptBuilder>,
        from: interface::Language,
        to: &interface::Language,
    ) -> Result<String, interface::error::Error> {
        let mut arr = self.translate_vec(&vec![query.to_owned()], None, from, to)?;
        Ok(arr.remove(0))
    }

    fn translate_vec(
        &mut self,
        query: &[String],
        _: Option<PromptBuilder>,
        from: interface::Language,
        to: &interface::Language,
    ) -> Result<Vec<String>, interface::error::Error> {
        let from = from.to_m2m100().ok_or(Error::UnknownLanguage(from))?;
        let to = to.to_m2m100().ok_or(Error::UnknownLanguage(to.clone()))?;
        *self.from.lock().unwrap() = from.to_owned();
        let model = self.load()?;
        let trans = model
            .translate_batch_with_target_prefix(
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
            )
            .map_err(Error::CTranslator)?;
        Ok(trans.into_iter().map(|v| v.0).collect())
    }
}

impl ModelLoad for M2M100Translator {
    type T = ct2rs::Translator<MyTokenizer>;

    fn loaded(&self) -> bool {
        self.loaded_models.is_some()
    }

    fn get_model(&mut self) -> Option<&mut Self::T> {
        self.loaded_models.as_mut()
    }

    fn reload(&mut self) -> Result<&mut Self::T, ModelLoadError> {
        let model_name = match self.size {
            Size::Small => "418M",
            Size::Large => "1.2B",
        };
        let model = self.download_model(model_name, &format!("{}/model.bin", model_name))?;
        let path = self.download_model("spm", "sentencepiece.bpe.model")?;
        let from = Arc::new(Mutex::new("".to_string()));
        let tokenizer = MyTokenizer::new(interface::tokenizer::SentenceTokenizer::new(path), from);
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
        )
        .map_err(Error::CTranslator)
        .unwrap();

        self.loaded_models = Some(v);
        Ok(self.loaded_models.as_mut().unwrap())
    }
}

impl Model for M2M100Translator {
    impl_model_load_helpers!("translator", "M2M100");

    fn models(&self) -> std::collections::HashMap<&'static str, interface_model::ModelSource> {
        hashmap! {
            "418M" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/m2m100-418m/418M.tar.gz",
                hash: "b232109dd3c8e7053f35820fdc7c5bdc64e752096e9c23b58ff70356fe29c1ed",
            },
            "1.2B" => ModelSource {
                url: "https://huggingface.co/FrederikSch/M2M100-1.2B-ct2/resolve/main/1.2B.tar.gz?download=true",
                hash: "5d5262729371349f4b13e5aaa988be6f49715fa69a765fd1029966727665fdbb",
            },
            "spm" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/m2m100-418m/sentencepiece.bpe.model",
                hash: "d8f7c76ed2a5e0822be39f0a4f95a55eb19c78f4593ce609e2edbc2aea4d380a"
            }
        }
    }

    fn unload(&mut self) {
        self.loaded_models = None;
    }
}

#[cfg(test)]
mod tests {
    use env_logger::Env;
    use interface::Language;

    use super::*;

    #[test]
    fn convert_langs() {
        let langs = [
            "__af__", "__am__", "__ar__", "__ast__", "__az__", "__ba__", "__be__", "__bg__",
            "__bn__", "__br__", "__bs__", "__ca__", "__ceb__", "__cs__", "__cy__", "__da__",
            "__de__", "__el__", "__en__", "__es__", "__et__", "__fa__", "__ff__", "__fi__",
            "__fr__", "__fy__", "__ga__", "__gd__", "__gl__", "__gu__", "__ha__", "__he__",
            "__hi__", "__hr__", "__ht__", "__hu__", "__hy__", "__id__", "__ig__", "__ilo__",
            "__is__", "__it__", "__ja__", "__jv__", "__ka__", "__kk__", "__km__", "__kn__",
            "__ko__", "__lb__", "__lg__", "__ln__", "__lo__", "__lt__", "__lv__", "__mg__",
            "__mk__", "__ml__", "__mn__", "__mr__", "__ms__", "__my__", "__ne__", "__nl__",
            "__no__", "__ns__", "__oc__", "__or__", "__pa__", "__pl__", "__ps__", "__pt__",
            "__ro__", "__ru__", "__sd__", "__si__", "__sk__", "__sl__", "__so__", "__sq__",
            "__sr__", "__ss__", "__su__", "__sv__", "__sw__", "__ta__", "__th__", "__tl__",
            "__tn__", "__tr__", "__uk__", "__ur__", "__uz__", "__vi__", "__wo__", "__xh__",
            "__yi__", "__yo__", "__zh__", "__zu__",
        ];
        for lang in langs {
            Language::from_m2m100(lang).expect(lang);
        }
    }
    #[test]
    fn test_load() {
        let mut m2m100 = M2M100Translator::new(false, ComputeType::DEFAULT, Size::Small);
        assert!(m2m100.load().is_ok());
        assert!(m2m100.loaded());
    }

    #[test]
    fn test_translate() {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
        let mut m2m100 = M2M100Translator::new(false, ComputeType::INT8, Size::Large);
        let input_ja = vec![
            "明日は雨が降るかもしれません。".to_string(),
            "彼はその問題について深く考えている。".to_string(),
            "このソフトウェアは非常に使いやすいです。".to_string(),
        ];

        let out = m2m100
            .translate_vec(&input_ja, None, Language::Japanese, &Language::English)
            .expect("Translation failed");
        assert_eq!(
            out,
            vec![
                "It may rain tomorrow.".to_owned(),
                "He is thinking deeply about the problem.".to_owned(),
                "This software is very easy to use.".to_owned()
            ]
        );

        let input_en = vec![
            "The meeting has been postponed until next week.".to_string(),
            "She quickly realized that something was wrong.".to_string(),
            "Artificial intelligence is changing the world rapidly.".to_string(),
        ];
        let out = m2m100
            .translate_vec(&input_en, None, Language::English, &Language::Japanese)
            .expect("Translation failed");
        assert_eq!(
            out,
            vec![
                "会議は次の週まで延期された。".to_owned(),
                "彼は素早く、それが何らかの悪意であることを悟った。".to_owned(),
                "人工知能は世界を急速に変えています。".to_owned()
            ]
        );
    }
}
