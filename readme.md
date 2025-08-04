# Getting Started
Offline translators are using blocking. Online async
```rs
pub use aio_translator::Translator;
let cuda = true;
let mut t = aio_translator::SugoiTranslator::new(cuda, aio_translator::ComputeType::DEFAULT);
t.translator_mut().as_blocking().unwrap()
    .translate_vec(
        &["Hello World".to_owned()],
        None,
        aio_translator::Language::Japanese,
        &aio_translator::Language::English,
    )
    .unwrap();
```
# Languages
- [Table](crates/lang-generator/src/map.md)

# Modules
Symbols
- `*`: No translation test
- `-`: No online language map test

## Offline
- [x] sugoi
- [x] jparacrawl
- [x] m2m100
- [x] mbart50
- [x] nllb
- [ ] qwen2

## Api
- [x] google
- [x] mymemory
- [x] deepl
- [x] baidu *-
- [x] caiyun *-
- [x] youdao *-

- [ ] groq
- [ ] deepseek
- [ ] chatgpt
- [ ] gemini

## Scraped
- [x] papago


## Detector
- [x] langid
- [x] whatlang
- [x] langua
