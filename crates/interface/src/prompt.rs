use std::collections::HashMap;

type ContentBuilder =
    fn(from: &str, to: &str, queries: &[String], data: PromptData) -> Option<String>;
pub struct PromptBuilder {
    pd: PromptData,
    msgs: Vec<Message>,
}

impl PromptBuilder {
    pub fn new(pd: PromptData) -> Self {
        let mut msgs = vec![Message::chat_system_template()];
        msgs.extend(Message::chat_sample());
        msgs.push(Message::main());
        Self { pd, msgs }
    }

    pub fn build(&self, from: &str, to: &str, queries: &[String]) -> Vec<(String, String)> {
        todo!()
    }
}

pub struct Message {
    role: Role,
    content_builder: ContentBuilder,
}

impl Message {
    pub fn chat_system_template() -> Self {
        fn content_builder(_: &str, to: &str, _: &[String], data: PromptData) -> Option<String> {
            Some(data.chat_system_template.replace("{to_lang}", to))
        }
        Self {
            role: Role::System,
            content_builder,
        }
    }

    pub fn chat_sample() -> Vec<Self> {
        fn content_builder1(_: &str, to: &str, _: &[String], data: PromptData) -> Option<String> {
            data.chat_sample.get(to)?.get(0).cloned()
        }
        fn content_builder2(_: &str, to: &str, _: &[String], data: PromptData) -> Option<String> {
            data.chat_sample.get(to)?.get(1).cloned()
        }
        vec![
            Self {
                role: Role::User,
                content_builder: content_builder1,
            },
            Self {
                role: Role::Assistant,
                content_builder: content_builder2,
            },
        ]
    }

    pub fn main() -> Self {
        fn content_builder(
            _: &str,
            to_lang: &str,
            queries: &[String],
            _: PromptData,
        ) -> Option<String> {
            let mut prompt = vec![format!(
                "Translate into {to_lang} and keep the original format.\n\nOriginal:"
            )];
            for (i, query) in queries.iter().enumerate() {
                prompt.push(format!("\n<|{}|>{query}", i + 1));
            }
            Some(prompt.join(""))
        }

        Self {
            role: Role::User,
            content_builder,
        }
    }
}

pub struct PromptData {
    chat_system_template: String,
    chat_sample: HashMap<String, Vec<String>>,
}

enum Role {
    System,
    User,
    Assistant,
}
