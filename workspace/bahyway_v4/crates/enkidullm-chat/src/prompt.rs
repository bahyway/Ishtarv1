//! prompt.rs — Sovereign prompt builder for DeepSeek-Coder chat template.
//!
//! DeepSeek-Coder chat format:
//!   <|begin_of_sentence|>system\n{system}\n<|end_of_sentence|>
//!   <|begin_of_sentence|>user\n{user}\n<|end_of_sentence|>
//!   <|begin_of_sentence|>assistant\n
#![forbid(unsafe_code)]

/// TamuzAI sovereign system prompt — BahyWay MetaEngine identity.
pub const DEEPSEEK_SYSTEM: &str = "\
You are TamuzAI (𒀭𒁕𒍪), the sovereign offline MetaEngine of BahyWay.Ecosystem v4.0. \
You are named after Tammuz (Dumuzi), the Sumerian shepherd-god who descends into the \
Underworld of code and returns with knowledge.

You know every crate, every ADR, and every sovereign law of BahyWay.Ecosystem v4.0:
- 79 Rust crates across 12 layers
- 10 Sovereign Laws (no external DB, append-only, no assessments in KAKI nucleus)
- KAKI: 16-byte sovereign identity (uuid_hash|tribe_id|type|role|seq_counter|timestamp|checksum)
- H(P) = 1/(1+√Σwᵢ(Pᵢ−Tᵢ)²) — quality equation, B11 = round(H(P)×240), QUALITY_DIVISOR=240
- ColorID(RGB) is EAV attribute — NOT in KAKI bytes
- Languages: AkkadianAOL (.akk), HeptaScript (.hepta), Way (.way), TemplateEngine (.tmpl)
- Triple-O: Orbits-Oriented Ontology — sovereign particles, not objects

You answer in precise technical Rust-focused language. \
You never suggest external dependencies. \
You never suggest PostgreSQL, Redis, Kafka, or any banned technology. \
You always respect the 10 Sovereign Laws. \
When asked about code, you write pure Rust with #![forbid(unsafe_code)].";

/// Supported chat template formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatTemplate {
    DeepSeek,
    Qwen2,
    LLaMA3,
    Generic,
}

impl ChatTemplate {
    pub fn from_model_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.contains("deepseek")  { Self::DeepSeek }
        else if lower.contains("qwen") { Self::Qwen2 }
        else if lower.contains("llama-3") || lower.contains("llama3") { Self::LLaMA3 }
        else { Self::Generic }
    }
}

/// Builds formatted prompts for different model chat templates.
pub struct PromptBuilder {
    template: ChatTemplate,
    system:   String,
}

impl PromptBuilder {
    pub fn new(template: ChatTemplate, system: &str) -> Self {
        Self { template, system: system.to_string() }
    }

    /// Build a default TamuzAI prompt builder.
    pub fn tamuzai() -> Self {
        Self::new(ChatTemplate::DeepSeek, DEEPSEEK_SYSTEM)
    }

    /// Build a full prompt from conversation history.
    /// history: [(role, content)] — "user" or "assistant"
    pub fn build(&self, history: &[(&str, &str)], new_user_msg: &str) -> String {
        match self.template {
            ChatTemplate::DeepSeek => self.build_deepseek(history, new_user_msg),
            ChatTemplate::Qwen2    => self.build_qwen2(history, new_user_msg),
            ChatTemplate::LLaMA3   => self.build_llama3(history, new_user_msg),
            ChatTemplate::Generic  => self.build_generic(history, new_user_msg),
        }
    }

    fn build_deepseek(&self, history: &[(&str, &str)], new_user: &str) -> String {
        let mut prompt = format!(
            "<|begin_of_sentence|>system\n{}\n<|end_of_sentence|>\n",
            self.system
        );
        for (role, content) in history {
            match *role {
                "user" => prompt.push_str(&format!(
                    "<|begin_of_sentence|>user\n{content}\n<|end_of_sentence|>\n"
                )),
                "assistant" => prompt.push_str(&format!(
                    "<|begin_of_sentence|>assistant\n{content}\n<|end_of_sentence|>\n"
                )),
                _ => {}
            }
        }
        prompt.push_str(&format!(
            "<|begin_of_sentence|>user\n{new_user}\n<|end_of_sentence|>\n\
             <|begin_of_sentence|>assistant\n"
        ));
        prompt
    }

    fn build_qwen2(&self, history: &[(&str, &str)], new_user: &str) -> String {
        let mut prompt = format!(
            "<|im_start|>system\n{}\n<|im_end|>\n",
            self.system
        );
        for (role, content) in history {
            prompt.push_str(&format!("<|im_start|>{role}\n{content}\n<|im_end|>\n"));
        }
        prompt.push_str(&format!(
            "<|im_start|>user\n{new_user}\n<|im_end|>\n<|im_start|>assistant\n"
        ));
        prompt
    }

    fn build_llama3(&self, history: &[(&str, &str)], new_user: &str) -> String {
        let mut prompt = format!(
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\
             {}\n<|eot_id|>",
            self.system
        );
        for (role, content) in history {
            prompt.push_str(&format!(
                "<|start_header_id|>{role}<|end_header_id|>\n{content}<|eot_id|>"
            ));
        }
        prompt.push_str(&format!(
            "<|start_header_id|>user<|end_header_id|>\n{new_user}<|eot_id|>\
             <|start_header_id|>assistant<|end_header_id|>\n"
        ));
        prompt
    }

    fn build_generic(&self, history: &[(&str, &str)], new_user: &str) -> String {
        let mut prompt = format!("System: {}\n\n", self.system);
        for (role, content) in history {
            let r = if *role == "user" { "Human" } else { "Assistant" };
            prompt.push_str(&format!("{r}: {content}\n"));
        }
        prompt.push_str(&format!("Human: {new_user}\nAssistant: "));
        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deepseek_template_detected() {
        assert_eq!(ChatTemplate::from_model_name("deepseek-coder-6.7b"), ChatTemplate::DeepSeek);
    }

    #[test]
    fn qwen_template_detected() {
        assert_eq!(ChatTemplate::from_model_name("Qwen2.5-Coder-7B"), ChatTemplate::Qwen2);
    }

    #[test]
    fn deepseek_prompt_contains_system() {
        let builder = PromptBuilder::tamuzai();
        let prompt = builder.build(&[], "What is KAKI?");
        assert!(prompt.contains("TamuzAI"));
        assert!(prompt.contains("What is KAKI?"));
        assert!(prompt.contains("<|begin_of_sentence|>"));
    }

    #[test]
    fn deepseek_prompt_contains_history() {
        let builder = PromptBuilder::tamuzai();
        let history = vec![("user", "hello"), ("assistant", "hi")];
        let prompt = builder.build(&history, "next question");
        assert!(prompt.contains("hello"));
        assert!(prompt.contains("hi"));
        assert!(prompt.contains("next question"));
    }

    #[test]
    fn system_prompt_contains_sovereign_laws() {
        assert!(DEEPSEEK_SYSTEM.contains("KAKI"));
        assert!(DEEPSEEK_SYSTEM.contains("Sovereign Laws"));
        assert!(DEEPSEEK_SYSTEM.contains("240"));
        assert!(DEEPSEEK_SYSTEM.contains("forbid(unsafe_code)"));
    }

    #[test]
    fn qwen2_prompt_format_correct() {
        let builder = PromptBuilder::new(ChatTemplate::Qwen2, "test system");
        let prompt = builder.build(&[], "user message");
        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("<|im_start|>assistant"));
    }
}
