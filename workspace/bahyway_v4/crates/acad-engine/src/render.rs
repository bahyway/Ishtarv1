//! Static HTML5 emission per target domain. TemplateEngine
//! (.tmpl) replaces this string builder post-gate; the OUTPUT
//! contract (static HTML5, MathML, pre-highlighted, canonical
//! link) is what this module seals.

use crate::{Block, Lecture, SiteDomain};

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn render_lecture(l: &Lecture, domain: SiteDomain) -> String {
    let mut h = String::new();
    h += "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n";
    h += &format!(
        "<meta charset=\"utf-8\">\n<title>{}</title>\n",
        esc(&l.title)
    );
    if domain != SiteDomain::Bahyway {
        h += &format!("<link rel=\"canonical\" href=\"{}\">\n", l.canonical());
    }
    h += "<meta name=\"generator\" content=\"AcadEngine\">\n</head>\n<body>\n";
    h += &format!(
        "<header data-site=\"{}\"><h1>{}</h1>\n",
        domain.host(),
        esc(&l.title)
    );
    h += &format!(
        "<p class=\"address\">{:?} / {:?} / {}</p>\n",
        l.sector,
        l.category,
        esc(&l.era)
    );
    if let Some(e) = &l.superseded_by {
        h += &format!("<p class=\"superseded\">Superseded in era {}</p>\n", esc(e));
    }
    h += "</header>\n<section class=\"w5h2\">\n";
    for (label, text) in [
        ("WHO", &l.w5h2.who),
        ("WHAT", &l.w5h2.what),
        ("WHEN", &l.w5h2.when),
        ("WHERE", &l.w5h2.where_),
        ("WHY", &l.w5h2.why),
        ("HOW", &l.w5h2.how),
        ("HOW MUCH", &l.w5h2.how_much),
    ] {
        if !text.is_empty() {
            h += &format!("<h2>{label}</h2>\n<p>{}</p>\n", esc(text));
        }
    }
    h += "</section>\n<section class=\"body\">\n";
    for b in &l.blocks {
        match b {
            Block::Prose(p) => h += &format!("<p>{}</p>\n", esc(p)),
            Block::MathMl(m) => h += &format!("<div class=\"math\">{m}</div>\n"),
            Block::Code { lang, source } => {
                h += &format!(
                    "<pre class=\"code lang-{}\"><code>{}</code></pre>\n",
                    esc(lang),
                    esc(source)
                );
            }
            Block::Video { src, caption } => {
                h += &format!(
                    "<figure><video controls src=\"{}\"></video><figcaption>{}</figcaption></figure>\n",
                    esc(src), esc(caption));
            }
            Block::Reference { cite, note } => {
                h += &format!(
                    "<p class=\"ref\"><cite>{}</cite> — {}</p>\n",
                    esc(cite),
                    esc(note)
                );
            }
        }
    }
    h += "</section>\n</body>\n</html>\n";
    h
}

#[cfg(test)]
mod tests {
    use super::render_lecture;
    use crate::*;

    fn founding() -> Lecture {
        Lecture {
            kaki_hex: "0011223344556677".into(),
            title: "The Puhu Law".into(),
            sector: Sector::Mummu,
            category: Category::Law,
            era: "Gudea10".into(),
            w5h2: W5h2 {
                who: "TOP Algebra; the Tribe; the shar puhi".into(),
                why: "The Tribe is the throne, not the king".into(),
                ..Default::default()
            },
            blocks: vec![Block::Reference {
                cite: "PH-002".into(),
                note: "the sealed tablet".into(),
            }],
            superseded_by: None,
            declared_domains: vec![],
        }
    }

    #[test]
    fn routing_law_holds() {
        // MUMMU law -> bahyway only by default
        assert_eq!(founding().target_domains(), vec![SiteDomain::Bahyway]);
        // DUBSAR lecture -> bahyway + heptascript
        let mut l = founding();
        l.sector = Sector::Dubsar;
        assert!(l.target_domains().contains(&SiteDomain::Heptascript));
        // ADAD lecture -> bahyway + beemdm
        l.sector = Sector::Adad;
        assert!(l.target_domains().contains(&SiteDomain::Beemdm));
        // Declaration adds a face without duplicating
        l.declared_domains = vec![SiteDomain::Beemdm, SiteDomain::Heptascript];
        let t = l.target_domains();
        assert_eq!(t.iter().filter(|d| **d == SiteDomain::Beemdm).count(), 1);
        assert!(t.contains(&SiteDomain::Heptascript));
    }

    #[test]
    fn mirror_declares_canonical() {
        let mut l = founding();
        l.sector = Sector::Dubsar;
        let mirror = render_lecture(&l, SiteDomain::Heptascript);
        assert!(mirror.contains("rel=\"canonical\""));
        assert!(mirror.contains("www.bahyway.com"));
        let main = render_lecture(&l, SiteDomain::Bahyway);
        assert!(!main.contains("rel=\"canonical\""));
    }
}
