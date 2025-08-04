use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    path::Path,
};

use rust_tokenizers::tokenizer::{SentencePieceTokenizer, Tokenizer as _};

pub struct SentenceTokenizer {
    spp: SentencePieceTokenizer,
}

impl SentenceTokenizer {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let spp = SentencePieceTokenizer::from_file(path, false).unwrap();
        Self { spp }
    }
}

impl ct2rs::Tokenizer for SentenceTokenizer {
    fn encode(&self, input: &str) -> anyhow::Result<Vec<String>> {
        let mut tokens = self.spp.tokenize(input);
        tokens.push("</s>".to_owned());
        Ok(tokens)
    }

    fn decode(&self, tokens: Vec<String>) -> anyhow::Result<String> {
        Ok(self.spp.convert_tokens_to_string(tokens).trim().to_owned())
    }
}

pub struct Dict {
    names: DictDefaults,
    bos_index: usize,
    pad_index: usize,
    pub eos_index: usize,
    unk_index: usize,
    nspecial: usize,
    indecies: HashMap<String, usize>,
    symbols: Vec<String>,
}

fn add_symbol(
    map: &mut HashMap<String, usize>,
    symbols: &mut Vec<String>,
    count: &mut Vec<usize>,
    word: String,
    n: usize,
    overr: bool,
) -> usize {
    if map.contains_key(&word) && !overr {
        let idx = *map.get(&word).unwrap();
        count[idx] = count[idx] + n;
        idx
    } else {
        let idx = map.len();
        map.insert(word.clone(), idx);
        count.push(n);
        symbols.push(word);
        idx
    }
}

pub struct DictDefaults {
    bos: String,
    pad: String,
    eos: String,
    unk: String,
}

impl Default for DictDefaults {
    fn default() -> Self {
        Self {
            bos: "<s>".to_owned(),
            pad: "<pad>".to_owned(),
            eos: "</s>".to_owned(),
            unk: "<unk>".to_owned(),
        }
    }
}

fn line_tokenizer(line: &str) -> Vec<&str> {
    line.split_whitespace().collect()
}

impl Dict {
    pub fn new<P: AsRef<Path>>(s: P, names: DictDefaults) -> Self {
        let s = read_to_string(s).unwrap();
        let indices_start_line = 0;
        let mut indecies = HashMap::new();
        let mut symbols = Vec::new();
        let mut countt = Vec::new();
        let bos_index = add_symbol(
            &mut indecies,
            &mut symbols,
            &mut countt,
            names.bos.clone(),
            1,
            false,
        );
        let pad_index = add_symbol(
            &mut indecies,
            &mut symbols,
            &mut countt,
            names.pad.clone(),
            1,
            false,
        );
        let eos_index = add_symbol(
            &mut indecies,
            &mut symbols,
            &mut countt,
            names.eos.clone(),
            1,
            false,
        );
        let unk_index = add_symbol(
            &mut indecies,
            &mut symbols,
            &mut countt,
            names.unk.clone(),
            1,
            false,
        );

        let nspecial = symbols.len();
        for line in s.lines().skip(indices_start_line) {
            let (mut line, mut field) = line.trim_end().rsplit_once(" ").unwrap();
            let overwrite = if field == "#fairseq:overwrite" {
                (line, field) = line.trim_end().rsplit_once(" ").unwrap();
                true
            } else {
                false
            };
            let count = field.parse::<usize>().unwrap();
            let word = line;
            if indecies.contains_key(word) && !overwrite {
                panic!(
                    "Duplicate word found when loading Dictionary: '{word}'. \
                    Duplicate words can overwrite earlier ones by adding the \
                    #fairseq:overwrite flag at the end of the corresponding row \
                    in the dictionary file. If using the Camembert model, please \
                    download an updated copy of the model file."
                )
            }
            add_symbol(
                &mut indecies,
                &mut symbols,
                &mut countt,
                word.to_owned(),
                count,
                overwrite,
            );
        }
        Self {
            names,
            bos_index,
            pad_index,
            eos_index,
            unk_index,
            nspecial,
            indecies,
            symbols,
        }
    }
    fn index(&self, word: &str) -> Option<usize> {
        self.indecies.get(word).copied()
    }

    pub fn encode_line(&self, line: &str, append_eos: bool) -> Vec<usize> {
        let words = line_tokenizer(line);
        let mut ids: Vec<usize> = words.into_iter().map(|v| self.index(v).unwrap()).collect();
        if append_eos {
            ids.push(self.eos_index);
        }
        ids
    }

    pub fn string(
        &self,
        ids: Vec<usize>,
        extra_symbols_to_ignore: Option<HashSet<usize>>,
        include_eos: bool,
        unk_string: Option<String>,
        escape_unk: bool,
    ) -> Vec<String> {
        let mut extra_symbols_to_ignore = extra_symbols_to_ignore.unwrap_or_default();
        if !include_eos {
            extra_symbols_to_ignore.insert(self.eos_index);
        }
        let token_string = |i: usize| {
            if i == self.unk_index {
                if unk_string.is_some() {
                    unk_string.clone().unwrap()
                } else {
                    self.unk_string(escape_unk)
                }
            } else {
                self.symbols[i].clone()
            }
        };
        extra_symbols_to_ignore.insert(self.bos_index);
        ids.into_iter()
            .filter(|v| !extra_symbols_to_ignore.contains(v))
            .map(|i| token_string(i))
            .collect()
    }

    fn unk_string(&self, escape: bool) -> String {
        if escape {
            format!("<{}>", self.names.unk)
        } else {
            self.names.unk.clone()
        }
    }
}
