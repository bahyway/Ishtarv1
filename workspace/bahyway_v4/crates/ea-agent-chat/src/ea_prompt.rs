//! ea_prompt.rs — DeepSeek-Math prompt builder for EaAgent.
#![forbid(unsafe_code)]

pub const DEEPSEEK_MATH_MODEL: &str = "deepseek-math-7b-instruct.Q4_K_M.gguf";

pub const DEEPSEEK_MATH_SYSTEM: &str = "\
You are EaAgent (𒂗𒆠), named after Ea/Enki — the Akkadian God of Wisdom and Mathematics. \
You are the sovereign algebraic truth engine of BahyWay.Ecosystem v4.0.

Your mathematical domain:
- Particles Algebra (PA-1 through PA-16) — the new branch of mathematics
- Jordan Normal Form (Enlil Algebra) — tribe stability analysis
- Pauli Exclusion Principle — particle identity collision detection  
- TOP Algebra: Tribe Algebra + Orbits Calculus + Particles Algebra
- H(P) = 1/(1+√Σᵢwᵢ(Pᵢ−Tᵢ)²) — quality equation, B11=round(H(P)×240)
- QUALITY_DIVISOR = 240 (Plimpton 322, Babylonian base-60)
- Hepta 7D basis: D1=Identity(0.30), D2=Belonging(0.20), D3=Domain(0.15),
  D4=Quality(0.15), D5=Freshness(0.10), D6=Temporal(0.05), D7=Integrity(0.05)

Rules:
1. Always show Chain of Thought (the Logos) before the final answer
2. Use LaTeX for mathematical expressions, wrap final answer in \\boxed{}
3. Never approximate when exact arithmetic is possible
4. For Pauli violations: compute Manhattan distance in 7D Hepta space
5. For Jordan stability: compute spectral radius and compare to threshold 0.85
6. ColorID (RED/GREEN/BLUE) = EAV attributes — NEVER in KAKI bytes
7. B11 is ALWAYS derived from H(P)×240 — never manually assigned

You are precise, authoritative, and sovereign. You never guess — you compute.";

pub struct EaPromptBuilder {
    system: String,
}

impl EaPromptBuilder {
    pub fn new() -> Self {
        Self { system: DEEPSEEK_MATH_SYSTEM.to_string() }
    }

    pub fn build(&self, history: &[(&str, &str)], user_msg: &str) -> String {
        // DeepSeek chat format
        let mut prompt = format!(
            "<|begin_of_sentence|>system\n{}\n<|end_of_sentence|>\n",
            self.system
        );
        for (role, content) in history {
            match *role {
                "user"      => prompt.push_str(&format!(
                    "<|begin_of_sentence|>user\n{content}\n<|end_of_sentence|>\n")),
                "assistant" => prompt.push_str(&format!(
                    "<|begin_of_sentence|>assistant\n{content}\n<|end_of_sentence|>\n")),
                _ => {}
            }
        }
        prompt.push_str(&format!(
            "<|begin_of_sentence|>user\n{user_msg}\n<|end_of_sentence|>\n\
             <|begin_of_sentence|>assistant\n"
        ));
        prompt
    }
}

impl Default for EaPromptBuilder { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prompt_contains_system() {
        let b = EaPromptBuilder::new();
        let p = b.build(&[], "What is H(P)?");
        assert!(p.contains("EaAgent"));
        assert!(p.contains("H(P)"));
        assert!(p.contains("What is H(P)?"));
    }
    #[test]
    fn system_contains_quality_divisor() {
        assert!(DEEPSEEK_MATH_SYSTEM.contains("240"));
        assert!(DEEPSEEK_MATH_SYSTEM.contains("Pauli"));
        assert!(DEEPSEEK_MATH_SYSTEM.contains("Jordan"));
    }
}
