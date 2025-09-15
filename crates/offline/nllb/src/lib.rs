use std::sync::{Arc, Mutex};

use aio_translator_interface::{
    BlockingTranslator, Language, Model, Translator, TranslatorMutTrait, TranslatorTrait,
    error::{self, Error},
    prompt::PromptBuilder,
    tokenizer::SentenceTokenizer,
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

pub struct NLLBTranslator {
    loaded_models: Option<ct2rs::Translator<MyTokenizer>>,
    cuda: bool,
    compute_type: ComputeType,
    size: Size,
    from: Arc<Mutex<String>>,
}

pub enum Size {
    SmallDistilled,
    Base,
    Large,
}

impl NLLBTranslator {
    /// single_loaded will only allow one model to be loaded at a time.
    pub fn new(cuda: bool, compute_type: ComputeType, size: Size) -> Self {
        NLLBTranslator {
            compute_type,
            cuda,
            size,
            loaded_models: None,
            from: Arc::default(),
        }
    }
}

impl Translator for NLLBTranslator {
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

impl BlockingTranslator for NLLBTranslator {
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
        let from = from.to_nllb().ok_or(Error::UnknownLanguage(from))?;
        let to = to.to_nllb().ok_or(Error::UnknownLanguage(to.clone()))?;
        *self.from.lock().unwrap() = from.to_owned();

        let model = self.load().map_err(error::Error::ModelLoadError)?;

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

impl ModelLoad for NLLBTranslator {
    type T = ct2rs::Translator<MyTokenizer>;

    fn loaded(&self) -> bool {
        self.loaded_models.is_some()
    }

    fn get_model(&mut self) -> Option<&mut Self::T> {
        self.loaded_models.as_mut()
    }

    fn reload(&mut self) -> anyhow::Result<&mut Self::T> {
        let model_name = match self.size {
            Size::SmallDistilled => "600M-distilled",
            Size::Large => "3.3B",
            Size::Base => "1.3B",
        };
        let model = self.download_model(model_name, &format!("{}/model.bin", model_name))?;
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
        )
        .map_err(Error::CTranslator)
        .unwrap();

        self.loaded_models = Some(v);
        Ok(self.loaded_models.as_mut().unwrap())
    }
}

impl Model for NLLBTranslator {
    impl_model_load_helpers!("translator", "nllb");

    fn models(&self) -> std::collections::HashMap<&'static str, interface_model::ModelSource> {
        hashmap! {
            "600M-distilled" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/nllb-200-600m-distilled/600M-distilled.tar.gz",
                hash: "4eadd328098fa4737d8e48e550a9c6f9ce795892dead84acbd6f3999bc125038",
            },
            "1.3B" => ModelSource {
                url: "https://huggingface.co/FrederikSch/nllb-200-1.3B-ct2/resolve/main/1.3B.tar.gz?download=true",
                hash: "979b87827f5bd23c460f3c9e9fe3a3c387461b62b9b22b7afbf95e78b9c483ce",
            },
            "3.3B" => ModelSource {
                url: "https://huggingface.co/FrederikSch/nllb-200-3.3B/resolve/main/3.3B.tar.gz?download=true",
                hash: "095c1b78a8d7ecbfa8f20906737ebb0918dbeec4f32c16d2d9b2cf05a6f8e87e",
            },
            "spm" => ModelSource {
                url: "https://github.com/frederik-uni/aiotranslator/releases/download/nllb-200-600m-distilled/sentencepiece.bpe.model",
                hash: "14bb8dfb35c0ffdea7bc01e56cea38b9e3d5efcdcb9c251d6b40538e1aab555a",
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
            "ace_Arab", "ace_Latn", "acm_Arab", "acq_Arab", "aeb_Arab", "afr_Latn", "ajp_Arab",
            "aka_Latn", "amh_Ethi", "apc_Arab", "arb_Arab", "ars_Arab", "ary_Arab", "arz_Arab",
            "asm_Beng", "ast_Latn", "awa_Deva", "ayr_Latn", "azb_Arab", "azj_Latn", "bak_Cyrl",
            "bam_Latn", "ban_Latn", "bel_Cyrl", "bem_Latn", "ben_Beng", "bho_Deva", "bjn_Arab",
            "bjn_Latn", "bod_Tibt", "bos_Latn", "bug_Latn", "bul_Cyrl", "cat_Latn", "ceb_Latn",
            "ces_Latn", "cjk_Latn", "ckb_Arab", "crh_Latn", "cym_Latn", "dan_Latn", "deu_Latn",
            "dik_Latn", "dyu_Latn", "dzo_Tibt", "ell_Grek", "eng_Latn", "epo_Latn", "est_Latn",
            "eus_Latn", "ewe_Latn", "fao_Latn", "pes_Arab", "fij_Latn", "fin_Latn", "fon_Latn",
            "fra_Latn", "fur_Latn", "fuv_Latn", "gla_Latn", "gle_Latn", "glg_Latn", "grn_Latn",
            "guj_Gujr", "hat_Latn", "hau_Latn", "heb_Hebr", "hin_Deva", "hne_Deva", "hrv_Latn",
            "hun_Latn", "hye_Armn", "ibo_Latn", "ilo_Latn", "ind_Latn", "isl_Latn", "ita_Latn",
            "jav_Latn", "jpn_Jpan", "kab_Latn", "kac_Latn", "kam_Latn", "kan_Knda", "kas_Arab",
            "kas_Deva", "kat_Geor", "knc_Arab", "knc_Latn", "kaz_Cyrl", "kbp_Latn", "kea_Latn",
            "khm_Khmr", "kik_Latn", "kin_Latn", "kir_Cyrl", "kmb_Latn", "kon_Latn", "kor_Hang",
            "kmr_Latn", "lao_Laoo", "lvs_Latn", "lij_Latn", "lim_Latn", "lin_Latn", "lit_Latn",
            "lmo_Latn", "ltg_Latn", "ltz_Latn", "lua_Latn", "lug_Latn", "luo_Latn", "lus_Latn",
            "mag_Deva", "mai_Deva", "mal_Mlym", "mar_Deva", "min_Latn", "mkd_Cyrl", "plt_Latn",
            "mlt_Latn", "mni_Beng", "khk_Cyrl", "mos_Latn", "mri_Latn", "zsm_Latn", "mya_Mymr",
            "nld_Latn", "nno_Latn", "nob_Latn", "npi_Deva", "nso_Latn", "nus_Latn", "nya_Latn",
            "oci_Latn", "gaz_Latn", "ory_Orya", "pag_Latn", "pan_Guru", "pap_Latn", "pol_Latn",
            "por_Latn", "prs_Arab", "pbt_Arab", "quy_Latn", "ron_Latn", "run_Latn", "rus_Cyrl",
            "sag_Latn", "san_Deva", "sat_Beng", "scn_Latn", "shn_Mymr", "sin_Sinh", "slk_Latn",
            "slv_Latn", "smo_Latn", "sna_Latn", "snd_Arab", "som_Latn", "sot_Latn", "spa_Latn",
            "als_Latn", "srd_Latn", "srp_Cyrl", "ssw_Latn", "sun_Latn", "swe_Latn", "swh_Latn",
            "szl_Latn", "tam_Taml", "tat_Cyrl", "tel_Telu", "tgk_Cyrl", "tgl_Latn", "tha_Thai",
            "tir_Ethi", "taq_Latn", "taq_Tfng", "tpi_Latn", "tsn_Latn", "tso_Latn", "tuk_Latn",
            "tum_Latn", "tur_Latn", "twi_Latn", "tzm_Tfng", "uig_Arab", "ukr_Cyrl", "umb_Latn",
            "urd_Arab", "uzn_Latn", "vec_Latn", "vie_Latn", "war_Latn", "wol_Latn", "xho_Latn",
            "ydd_Hebr", "yor_Latn", "yue_Hant", "zho_Hans", "zho_Hant", "zul_Latn",
        ];
        for lang in langs {
            Language::from_nllb(lang).expect(lang);
        }
    }

    #[test]
    fn test_load() {
        let mut nllb = NLLBTranslator::new(false, ComputeType::DEFAULT, Size::SmallDistilled);
        assert!(nllb.load().is_ok());
        assert!(nllb.loaded());
    }

    #[test]
    fn test_translate() {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
        let mut nllb = NLLBTranslator::new(false, ComputeType::DEFAULT, Size::Large);
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
                "He is thinking deeply about the problem.".to_owned(),
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
                "会議は来週まで延期された.".to_owned(),
                "彼女はすぐに何かが間違っていたことに気づきました".to_owned(),
                "人工知能は 世界を急速に変化させています".to_owned()
            ]
        );
    }
}
