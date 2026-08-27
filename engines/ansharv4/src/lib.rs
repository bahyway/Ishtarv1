//! ANŠARV4 ENGINE 𒀭𒊹 — the whole sky holds the whole graph.
//! "Anšarv4 — šar kali šarrī of graphs: the graph of all graphs." (Ḫubullu epithet)
//! NL-001: gods name engines. Naming tablet: NL-001_Naming_Tablet_AnsharV4Engine.
pub mod export;
pub mod orbit;
pub mod piristu;
pub mod store;

#[cfg(test)]
mod tests {
    use crate::export::to_json;
    use crate::orbit::{orbit_layers, ride_path};
    use crate::piristu::{certify_monotone, sealed, visible_label};
    use crate::store::{AnsharV4Store, IngestVerdict};

    fn mini() -> AnsharV4Store {
        let mut s = AnsharV4Store::default();
        for l in [
            "NODE ALG|THEOREM|GL-ALG-002|PUBLIC|C",
            "NODE L_AGE|LAW|GL-AGE-001|PARTNER|C",
            "NODE huburu|TERM|huburu|PARTNER|C",
            "NODE zaqiqu|TERM|zaqiqu|DUBSAR|C",
            "NODE g_noi|CRATE|gula::noise|DUBSAR|AC",
            "NODE g_zaq|CRATE|gula::zaqiqu|DUBSAR|AC",
            "NODE a362|APP|Agu Crown|PARTNER|BC",
            "NODE a358|APP|Zaqiqu Veil|PARTNER|BC",
            "EDGE ALG|L_AGE|governs|harranu",
            "EDGE L_AGE|huburu|names|harranu",
            "EDGE L_AGE|zaqiqu|names|harranu",
            "EDGE huburu|g_noi|implemented_in|suqu",
            "EDGE zaqiqu|g_zaq|implemented_in|suqu",
            "EDGE g_noi|a362|exhibited_in|suqu",
            "EDGE g_zaq|a358|exhibited_in|suqu",
            "HYPER H_huburu|tribe|tribe of huburu|L_AGE,huburu,g_noi,a362",
        ] {
            assert_eq!(s.ingest_line(l), IngestVerdict::Accepted, "{l}");
        }
        s
    }

    #[test]
    fn ingest_refuses_duplicates_and_malformed() {
        let mut s = mini();
        assert!(matches!(
            s.ingest_line("NODE ALG|THEOREM|dup|PUBLIC|C"),
            IngestVerdict::RejectedDuplicate(_)));
        assert!(matches!(
            s.ingest_line("NODE broken"),
            IngestVerdict::RejectedMalformed(_)));
    }

    #[test]
    fn integrity_court_passes_and_catches() {
        let s = mini();
        assert!(s.integrity().is_ok());
        let mut bad = mini();
        bad.ingest_line("EDGE ghost|ALG|haunts|kin");
        assert!(bad.integrity().is_err(), "missing endpoint must be caught");
    }

    #[test]
    fn piristu_is_monotone_and_leaks_nothing() {
        let s = mini();
        assert!(certify_monotone(&s));
        let z = &s.nodes["zaqiqu"];
        assert!(sealed(z, 0) && sealed(z, 1) && !sealed(z, 2));
        assert_eq!(visible_label(z, 0), "𒁾 pirištu");
        let export_visitor = to_json(&s, 0);
        assert!(!export_visitor.contains("zaqiqu\",\"sealed\":false"));
        assert!(!export_visitor.contains("\"label\":\"zaqiqu\""),
            "the engine's export must not carry the sealed name");
        assert!(to_json(&s, 2).contains("\"label\":\"zaqiqu\""));
    }

    #[test]
    fn orbit_layers_span_the_terraces() {
        let s = mini();
        let l = orbit_layers(&s, "huburu");
        assert!(l["LAW"].contains("L_AGE"));
        assert!(l["CRATE"].contains("g_noi"));
        assert!(l["APP"].contains("a362"));
    }

    #[test]
    fn every_ride_reaches_the_plain() {
        let s = mini();
        for start in ["huburu", "g_noi", "L_AGE", "zaqiqu"] {
            let p = ride_path(&s, start);
            assert_eq!(p[0], "ALG", "{start} starts at summit");
            assert_eq!(s.nodes[p.last().unwrap()].ntype, "APP",
                "{start} ends at an app: {:?}", p);
        }
    }
}
