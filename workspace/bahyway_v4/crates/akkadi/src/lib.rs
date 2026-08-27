//! akkadi — Sovereign Internal Global Language (𒀭𒆳𒁺)
//!
//! Bahaa's sovereign language drawing from:
//!   - Akkadian (Semitic root structure, cuneiform script)
//!   - Arabic   (semantic richness, vowel patterns)
//!   - English  (global tooling documentation)
//!   - Sovereign coinages (new words for digital/sovereign concepts)

pub mod notebook;
pub use notebook::CellKind;

// ── AkkadiRoot ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AkkadiRoot {
    pub consonants: &'static str,
    pub semantic: &'static str,
    pub akkadian: Option<&'static str>,
    pub arabic: Option<&'static str>,
    pub glyph: &'static str,
}

// ── AkkadiWord ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AkkadiWord {
    pub akkadi: &'static str,
    pub glyph: &'static str,
    pub english: &'static str,
    pub arabic: &'static str,
    pub gram: &'static str,
    pub domain: AkkadiDomain,
    pub example: Option<&'static str>,
    pub sovereign: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AkkadiDomain {
    Sovereign,
    Nature,
    Society,
    Divine,
    Survival,
    Language,
    Temporal,
}

impl std::fmt::Display for AkkadiDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sovereign => write!(f, "sovereign"),
            Self::Nature => write!(f, "nature"),
            Self::Society => write!(f, "society"),
            Self::Divine => write!(f, "divine"),
            Self::Survival => write!(f, "survival"),
            Self::Language => write!(f, "language"),
            Self::Temporal => write!(f, "temporal"),
        }
    }
}

// ── Vocabulary ────────────────────────────────────────────────────────────────

pub const AKKADI_WORDS: &[AkkadiWord] = &[
    AkkadiWord {
        akkadi: "kaki",
        glyph: "𒆪𒆠",
        english: "key, identifier, seal",
        arabic: "مفتاح (miftah)",
        gram: "n.",
        domain: AkkadiDomain::Sovereign,
        example: Some("kaki-ilu — divine identifier"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "enkitu",
        glyph: "𒂗𒆠",
        english: "database, memory-house",
        arabic: "قاعدة البيانات (qa'idat al-bayanat)",
        gram: "n.",
        domain: AkkadiDomain::Sovereign,
        example: Some("enkitu-rabi — great database"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "dubsar",
        glyph: "𒁾𒊬",
        english: "scribe, data writer",
        arabic: "كاتب (katib)",
        gram: "n.",
        domain: AkkadiDomain::Sovereign,
        example: Some("dubsar-bahyway — BahyWay scribe"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "šatum",
        glyph: "𒊬",
        english: "to write, to encode",
        arabic: "كتب (kataba)",
        gram: "v.",
        domain: AkkadiDomain::Sovereign,
        example: Some("kaki šatum — to write the key"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "kunnu",
        glyph: "𒆪𒉌",
        english: "to verify, to confirm, to seal",
        arabic: "تحقق (tahaqaqa)",
        gram: "v.",
        domain: AkkadiDomain::Sovereign,
        example: Some("gilgamesh-kunnu — Gilgamesh-verified"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "zeritu",
        glyph: "𒍣𒀀",
        english: "zero-trust zone, secure boundary",
        arabic: "منطقة آمنة (mintaqat amina)",
        gram: "n.",
        domain: AkkadiDomain::Sovereign,
        example: Some("zeritu-šulmu — secure peace zone"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "bahyway",
        glyph: "𒁀𒄩",
        english: "the sovereign ecosystem, the way",
        arabic: "المنظومة السيادية (al-manzuma al-siyadiya)",
        gram: "prop.",
        domain: AkkadiDomain::Sovereign,
        example: Some("bahyway-rabi — the great BahyWay"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "giltu",
        glyph: "𒄀𒂂",
        english: "graph, network, web",
        arabic: "شبكة (shabaka)",
        gram: "n.",
        domain: AkkadiDomain::Sovereign,
        example: Some("giltu-napšatu — life network"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "raqu",
        glyph: "𒊕𒀀",
        english: "vector, direction, ray",
        arabic: "متجه (mutajjih)",
        gram: "n.",
        domain: AkkadiDomain::Sovereign,
        example: Some("raqu-kunnu — verified vector"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "larsatu",
        glyph: "𒌓𒀕𒆠",
        english: "sovereign phone, mobile sovereignty",
        arabic: "الهاتف السيادي (al-hatif al-siyadiy)",
        gram: "n.",
        domain: AkkadiDomain::Sovereign,
        example: Some("larsatu-zeritu — secure sovereign phone"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "mêtum",
        glyph: "𒀀𒁲",
        english: "pure water, safe water",
        arabic: "ماء نقي (ma' naqi)",
        gram: "n.",
        domain: AkkadiDomain::Survival,
        example: Some("mêtum-ṭābum — good/safe water"),
        sovereign: false,
    },
    AkkadiWord {
        akkadi: "ḫarānum",
        glyph: "𒄩𒀭",
        english: "safe route, path of survival",
        arabic: "طريق آمن (tariq amin)",
        gram: "n.",
        domain: AkkadiDomain::Survival,
        example: Some("ḫarānum-šulmu — peaceful safe route"),
        sovereign: false,
    },
    AkkadiWord {
        akkadi: "napāḫum",
        glyph: "𒀭𒁀",
        english: "to survive, to breathe, to persist",
        arabic: "يبقى (yabqa)",
        gram: "v.",
        domain: AkkadiDomain::Survival,
        example: Some("nišū napāḫū — the people survive"),
        sovereign: false,
    },
    AkkadiWord {
        akkadi: "qātum",
        glyph: "𒂗𒋾",
        english: "safe zone, refuge, shelter",
        arabic: "ملجأ (malja')",
        gram: "n.",
        domain: AkkadiDomain::Survival,
        example: Some("qātum-kīnum — reliable shelter"),
        sovereign: false,
    },
    AkkadiWord {
        akkadi: "errum",
        glyph: "𒂗𒊒",
        english: "danger, threat, red zone",
        arabic: "خطر (khatar)",
        gram: "n.",
        domain: AkkadiDomain::Survival,
        example: Some("errum-dannum — great danger"),
        sovereign: false,
    },
    AkkadiWord {
        akkadi: "šarrutu",
        glyph: "𒈗𒌓",
        english: "sovereignty, kingship, supreme authority",
        arabic: "سيادة (siyada)",
        gram: "n.",
        domain: AkkadiDomain::Society,
        example: Some("šarrutu-bahyway — BahyWay sovereignty"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "dīnum",
        glyph: "𒁲𒉌",
        english: "law, judgment, policy",
        arabic: "حكم (hukm)",
        gram: "n.",
        domain: AkkadiDomain::Society,
        example: Some("dīnum-zeritu — zero-trust law"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "kittum",
        glyph: "𒆪𒋾",
        english: "truth, justice, correctness",
        arabic: "حق (haqq)",
        gram: "n.",
        domain: AkkadiDomain::Society,
        example: Some("kittum u mīšarum — truth and justice"),
        sovereign: false,
    },
    AkkadiWord {
        akkadi: "ilūtu",
        glyph: "𒀭𒌓",
        english: "divinity, sovereignty, sacred power",
        arabic: "ألوهية (uluhiya)",
        gram: "n.",
        domain: AkkadiDomain::Divine,
        example: None,
        sovereign: false,
    },
    AkkadiWord {
        akkadi: "nēmequ",
        glyph: "𒉌𒄩",
        english: "wisdom, deep knowledge, AI insight",
        arabic: "حكمة (hikma)",
        gram: "n.",
        domain: AkkadiDomain::Divine,
        example: Some("nēmequ-enkitu — wisdom of the database"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "akkadi",
        glyph: "𒀭𒆳",
        english: "Akkadi (the sovereign language itself)",
        arabic: "الأكادية السيادية (al-akkadiya al-siyadiya)",
        gram: "prop.",
        domain: AkkadiDomain::Language,
        example: Some("akkadi lišānum-šulmu — Akkadi: the language of peace"),
        sovereign: true,
    },
    AkkadiWord {
        akkadi: "lišānum",
        glyph: "𒅆𒅗",
        english: "language, tongue, expression",
        arabic: "لغة (lugha)",
        gram: "n.",
        domain: AkkadiDomain::Language,
        example: Some("lišānum-akkadi — Akkadi language"),
        sovereign: false,
    },
];

// ── AkkadianAOL Tablet Source ─────────────────────────────────────────────────

pub fn akkadi_tablet_source() -> String {
    let mut src = String::new();
    src.push_str("-- Akkadi Language v4.0 — Sovereign Internal Global Language\n");
    src.push_str("-- Generated by akkadi crate — DO NOT EDIT MANUALLY\n\n");
    src.push_str("tablet akkadi_language {\n");
    src.push_str("    @version \"4.0\"\n");
    src.push_str("    @domain sovereign\n\n");

    for word in AKKADI_WORDS {
        src.push_str(&format!(
            "    word {} {{\n\
             \t\takkadi    = \"{}\"\n\
             \t\tenglish   = \"{}\"\n\
             \t\tarabic    = \"{}\"\n\
             \t\tgram      = \"{}\"\n\
             \t\tdomain    = \"{}\"\n",
            word.akkadi.replace('-', "_"),
            word.akkadi,
            word.english,
            word.arabic,
            word.gram,
            word.domain,
        ));
        if let Some(ex) = word.example {
            src.push_str(&format!("\t\texample   = \"{}\"\n", ex));
        }
        if word.sovereign {
            src.push_str("\t\t@seal\n");
        }
        src.push_str("    }\n\n");
    }
    src.push_str("}\n");
    src
}

// ── AkkadiEngine ──────────────────────────────────────────────────────────────

pub struct AkkadiEngine;

impl AkkadiEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn get(&self, akkadi: &str) -> Option<&'static AkkadiWord> {
        AKKADI_WORDS.iter().find(|w| w.akkadi == akkadi)
    }

    pub fn search_english(&self, query: &str) -> Vec<&'static AkkadiWord> {
        let q = query.to_lowercase();
        AKKADI_WORDS
            .iter()
            .filter(|w| w.english.to_lowercase().contains(&q))
            .collect()
    }

    pub fn sovereign_vocab(&self) -> Vec<&'static AkkadiWord> {
        AKKADI_WORDS.iter().filter(|w| w.sovereign).collect()
    }

    pub fn by_domain(&self, domain: AkkadiDomain) -> Vec<&'static AkkadiWord> {
        AKKADI_WORDS.iter().filter(|w| w.domain == domain).collect()
    }

    pub fn tablet_source(&self) -> String {
        akkadi_tablet_source()
    }

    pub fn word_count(&self) -> usize {
        AKKADI_WORDS.len()
    }
}

impl Default for AkkadiEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_akkadi_words_non_empty() {
        assert!(AKKADI_WORDS.len() >= 20);
    }

    #[test]
    fn test_akkadi_engine_get() {
        let e = AkkadiEngine::new();
        let w = e.get("kaki").unwrap();
        assert!(w.english.contains("key"));
        assert!(w.sovereign);
    }

    #[test]
    fn test_akkadi_search_english() {
        let results = AkkadiEngine::new().search_english("water");
        assert!(results.iter().any(|w| w.akkadi == "mêtum"));
    }

    #[test]
    fn test_akkadi_sovereign_vocab() {
        let sv = AkkadiEngine::new().sovereign_vocab();
        assert!(sv.iter().all(|w| w.sovereign));
    }

    #[test]
    fn test_akkadi_by_domain_survival() {
        let s = AkkadiEngine::new().by_domain(AkkadiDomain::Survival);
        assert!(s.iter().any(|w| w.akkadi == "mêtum"));
        assert!(s.iter().any(|w| w.akkadi == "qātum"));
    }

    #[test]
    fn test_akkadi_tablet_source() {
        let src = AkkadiEngine::new().tablet_source();
        assert!(src.contains("tablet akkadi_language {"));
        assert!(src.contains("word kaki {"));
        assert!(src.contains("@seal"));
    }

    #[test]
    fn test_akkadi_domain_display() {
        assert_eq!(AkkadiDomain::Sovereign.to_string(), "sovereign");
        assert_eq!(AkkadiDomain::Survival.to_string(), "survival");
    }

    #[test]
    fn test_akkadi_self_referential() {
        let w = AkkadiEngine::new().get("akkadi").unwrap();
        assert_eq!(w.domain, AkkadiDomain::Language);
    }
}
