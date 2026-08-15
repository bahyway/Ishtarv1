//! AcadEngine -- one EnkiDDB/EnkiMDB content store, three
//! public faces (see SPEC-AKD-001.md).

pub mod render;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sector { Apsu, Adad, Shedu, Mummu, Enkidu, Dubsar, Enlil }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Category { Engine, Structure, Era, Artifact, Pattern, Calculus, Law }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SiteDomain { Bahyway, Heptascript, Beemdm }

impl SiteDomain {
    pub fn host(self) -> &'static str {
        match self {
            SiteDomain::Bahyway => "www.bahyway.com",
            SiteDomain::Heptascript => "www.heptascript.com",
            SiteDomain::Beemdm => "www.beemdm.com",
        }
    }
}

#[derive(Default, Clone)]
pub struct W5h2 {
    pub who: String,
    pub what: String,
    pub when: String,
    pub where_: String,
    pub why: String,
    pub how: String,
    pub how_much: String,
}

#[derive(Clone)]
pub enum Block {
    Prose(String),
    MathMl(String),
    Code { lang: String, source: String },
    Video { src: String, caption: String },
    Reference { cite: String, note: String },
}

#[derive(Clone)]
pub struct Lecture {
    pub kaki_hex: String,
    pub title: String,
    pub sector: Sector,
    pub category: Category,
    pub era: String,
    pub w5h2: W5h2,
    pub blocks: Vec<Block>,
    pub superseded_by: Option<String>,
    /// Explicit extra targets beyond routing defaults.
    pub declared_domains: Vec<SiteDomain>,
}

impl Lecture {
    /// Routing law (SPEC-AKD-001): bahyway.com receives
    /// everything; sector defaults add the language and
    /// product faces; declarations add the rest.
    pub fn target_domains(&self) -> Vec<SiteDomain> {
        let mut t = vec![SiteDomain::Bahyway];
        match self.sector {
            Sector::Dubsar => t.push(SiteDomain::Heptascript),
            Sector::Adad => t.push(SiteDomain::Beemdm),
            _ => {}
        }
        for d in &self.declared_domains {
            if !t.contains(d) { t.push(*d); }
        }
        t
    }

    pub fn path(&self) -> String {
        format!("{:?}/{:?}/{}/{}.html",
            self.sector, self.category, self.era, self.kaki_hex)
            .to_lowercase()
    }

    /// Canonical URL -- one scholarly identity across faces.
    pub fn canonical(&self) -> String {
        format!("https://{}/{}", SiteDomain::Bahyway.host(), self.path())
    }
}
