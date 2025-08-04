use ct2rs::{Config, GenerationOptions, Tokenizer};
pub struct MyTokenizer {
    t: tokenizers::Tokenizer,
}

impl MyTokenizer {
    pub fn new() -> Self {
        let t = tokenizers::Tokenizer::from_file("/Users/frederik/code/rust/aio-translator/crates/offline/qwen2/tokenizer2/tokenizer.json").unwrap();
        Self { t }
    }
}
impl Tokenizer for MyTokenizer {
    fn encode(&self, input: &str) -> anyhow::Result<Vec<String>> {
        let v = self.t.encode(input, true).unwrap();
        Ok(v.get_tokens().to_vec())
    }

    fn decode(&self, tokens: Vec<String>) -> anyhow::Result<String> {
        todo!("{:?}", tokens)
    }
}

fn main() {
    let prompt = "<|im_start|>system\nYou are a helpful assistant.<|im_end|>\n<|im_start|>user\nHello<|im_end|>\n<|im_start|>assistant\n";

    // tokens = tokenizer.convert_ids_to_tokens(tokenizer.encode(prompt, add_special_tokens=False))
    // results = generator.generate_batch([tokens], max_length=100, sampling_temperature=0.7);
    let t = ct2rs::Generator::with_tokenizer(
        "/Users/frederik/code/rust/aio-translator/crates/offline/qwen2/2-7B-Instruct",
        MyTokenizer::new(),
        &Config::default(),
    )
    .unwrap();
    let data = t
        .generate_batch(
            &vec![prompt],
            &GenerationOptions {
                sampling_temperature: 0.7,
                ..Default::default()
            },
            None,
        )
        .unwrap();
    println!("{:?}", data.into_iter().map(|v| v.0).collect::<Vec<_>>())
}
