use aio_translator_interface::{Detector, Language as InterfaceLanguage};
use lingua_rs::{Language, LanguageDetector, LanguageDetectorBuilder};

pub struct LinguaDetector {
    d: LanguageDetector,
}

impl LinguaDetector {
    pub fn new() -> Self {
        let d = LanguageDetectorBuilder::from_all_languages().build();
        Self { d }
    }
}
impl Detector for LinguaDetector {
    fn detect_language(&self, s: &str) -> Option<InterfaceLanguage> {
        let lang = self.d.detect_language_of(s)?;
        Some(match lang {
            Language::Afrikaans => InterfaceLanguage::Afrikaans,
            Language::Albanian => InterfaceLanguage::Albanian,
            Language::Arabic => InterfaceLanguage::Arabic,
            Language::Armenian => InterfaceLanguage::Armenian,
            Language::Azerbaijani => InterfaceLanguage::Azerbaijani,
            Language::Basque => InterfaceLanguage::Basque,
            Language::Belarusian => InterfaceLanguage::Belarusian,
            Language::Bengali => InterfaceLanguage::Bengali,
            Language::Bokmal => InterfaceLanguage::NorwegianBokmål,
            Language::Bosnian => InterfaceLanguage::Bosnian,
            Language::Bulgarian => InterfaceLanguage::Bulgarian,
            Language::Catalan => InterfaceLanguage::Catalan,
            Language::Chinese => InterfaceLanguage::Chinese,
            Language::Croatian => InterfaceLanguage::Croatian,
            Language::Czech => InterfaceLanguage::Czech,
            Language::Danish => InterfaceLanguage::Danish,
            Language::Dutch => InterfaceLanguage::Dutch,
            Language::English => InterfaceLanguage::English,
            Language::Esperanto => InterfaceLanguage::Esperanto,
            Language::Estonian => InterfaceLanguage::Estonian,
            Language::Finnish => InterfaceLanguage::Finnish,
            Language::French => InterfaceLanguage::French,
            Language::Ganda => InterfaceLanguage::Ganda,
            Language::Georgian => InterfaceLanguage::Georgian,
            Language::German => InterfaceLanguage::German,
            Language::Greek => InterfaceLanguage::Greek,
            Language::Gujarati => InterfaceLanguage::Gujarati,
            Language::Hebrew => InterfaceLanguage::Hebrew,
            Language::Hindi => InterfaceLanguage::Hindi,
            Language::Hungarian => InterfaceLanguage::Hungarian,
            Language::Icelandic => InterfaceLanguage::Icelandic,
            Language::Indonesian => InterfaceLanguage::Indonesian,
            Language::Irish => InterfaceLanguage::Irish,
            Language::Italian => InterfaceLanguage::Italian,
            Language::Japanese => InterfaceLanguage::Japanese,
            Language::Kazakh => InterfaceLanguage::Kazakh,
            Language::Korean => InterfaceLanguage::Korean,
            Language::Latin => InterfaceLanguage::Latin,
            Language::Latvian => InterfaceLanguage::Latvian,
            Language::Lithuanian => InterfaceLanguage::Lithuanian,
            Language::Macedonian => InterfaceLanguage::Macedonian,
            Language::Malay => InterfaceLanguage::Malay,
            Language::Maori => InterfaceLanguage::Maori,
            Language::Marathi => InterfaceLanguage::Marathi,
            Language::Mongolian => InterfaceLanguage::Mongolian,
            Language::Nynorsk => InterfaceLanguage::NorwegianNynorsk,
            Language::Persian => InterfaceLanguage::Persian,
            Language::Polish => InterfaceLanguage::Polish,
            Language::Portuguese => InterfaceLanguage::Portuguese,
            Language::Punjabi => InterfaceLanguage::Punjabi,
            Language::Romanian => InterfaceLanguage::Romanian,
            Language::Russian => InterfaceLanguage::Russian,
            Language::Serbian => InterfaceLanguage::Serbian,
            Language::Shona => InterfaceLanguage::Shona,
            Language::Slovak => InterfaceLanguage::Slovak,
            Language::Slovene => InterfaceLanguage::Slovenian,
            Language::Somali => InterfaceLanguage::Somali,
            Language::Sotho => InterfaceLanguage::SouthernSotho,
            Language::Spanish => InterfaceLanguage::Spanish,
            Language::Swahili => InterfaceLanguage::Swahili,
            Language::Swedish => InterfaceLanguage::Swedish,
            Language::Tagalog => InterfaceLanguage::Tagalog,
            Language::Tamil => InterfaceLanguage::Tamil,
            Language::Telugu => InterfaceLanguage::Telugu,
            Language::Thai => InterfaceLanguage::Thai,
            Language::Tsonga => InterfaceLanguage::Tsonga,
            Language::Tswana => InterfaceLanguage::Tswana,
            Language::Turkish => InterfaceLanguage::Turkish,
            Language::Ukrainian => InterfaceLanguage::Ukrainian,
            Language::Urdu => InterfaceLanguage::Urdu,
            Language::Vietnamese => InterfaceLanguage::Vietnamese,
            Language::Welsh => InterfaceLanguage::Welsh,
            Language::Xhosa => InterfaceLanguage::Xhosa,
            Language::Yoruba => InterfaceLanguage::Yoruba,
            Language::Zulu => InterfaceLanguage::Zulu,
        })
    }
}
