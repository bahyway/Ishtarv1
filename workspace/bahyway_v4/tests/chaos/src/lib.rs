//! Chaos / adversarial tests — robustness under malformed input, boundary
//! conditions, and high-volume ingestion.

#[cfg(test)]
mod kaki_integrity {
    use bahyway_core::TribeId;
    use bahyway_crc::crc16;
    use enkidb_kaki::{KakiMinter, KakiRole};

    #[test]
    fn minted_kaki_checksum_is_valid() {
        let m = KakiMinter::new(TribeId::from_u16(0xBEEF));
        let kaki = m.mint_identity(0xDEAD_BABE, KakiRole::Zikru);
        let bytes = kaki.bytes();
        let computed = crc16(&bytes[0..14]);
        let stored = u16::from_be_bytes([bytes[14], bytes[15]]);
        assert_eq!(computed, stored, "KAKI CRC-16 must be self-consistent");
    }

    #[test]
    fn two_mints_produce_distinct_kakis() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let a = m.identity(KakiRole::Zikru);
        let b = m.identity(KakiRole::Zikru);
        // Counter in minter ensures uuid_hash differs
        assert_ne!(
            a.bytes()[0..4],
            b.bytes()[0..4],
            "uuid_hash bytes must differ between mints"
        );
    }

    #[test]
    fn tribe_id_is_immutable_in_kaki() {
        let m = KakiMinter::new(TribeId::from_u16(0x00FF));
        let kaki = m.mint_identity(0x1234_5678, KakiRole::Zikru);
        let bytes = kaki.bytes();
        // D2 = tribe_id at bytes [4..6]
        let tribe_in_kaki = u16::from_be_bytes([bytes[4], bytes[5]]);
        assert_eq!(
            tribe_in_kaki, 0x00FF,
            "tribe_id must be baked into KAKI at birth"
        );
    }

    #[test]
    fn mint_identity_with_explicit_hash_is_deterministic_in_hash() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        // Both mints use the same uuid_hash — only timestamp may differ
        let k1 = m.mint_identity(0xAAAA_AAAA, KakiRole::Zikru);
        let k2 = m.mint_identity(0xAAAA_AAAA, KakiRole::Zikru);
        // uuid_hash bytes must be identical
        assert_eq!(
            k1.uuid_hash(),
            k2.uuid_hash(),
            "same input must produce same uuid_hash"
        );
        // uuid_hash is FNV-1a derived — not the raw input (see mint.rs:120)
        // Both checksums must be valid
        assert!(k1.verify_checksum());
        assert!(k2.verify_checksum());
    }
}

#[cfg(test)]
mod boundary_conditions {
    use adad_gate::{AdadGate, ArrivalRecord};
    use bahyway_core::{ParticleState, TribeId};
    use enkidb_engine::EnkiDb;
    use enkidb_kaki::KakiRole;
    use musaru_security::check_sovereignty;
    use permanent_storage::PermanentStore;
    use story_engine::projection::{encode_state, ATTR_STATE};
    use template_library::civil_registry_template;
    use vgca_validation::validate;

    fn commit_particle(
        gate: &AdadGate,
        store: &mut PermanentStore,
        epoch: u32,
        state: ParticleState,
    ) {
        let record = ArrivalRecord {
            attrs: vec![(ATTR_STATE, encode_state(state).to_vec())],
            epoch,
            role: KakiRole::Zikru,
        };
        let gr = gate.ingest(record).unwrap();
        let sec = check_sovereignty(gate.tribe_id(), &gr.particle);
        assert!(sec.is_approved());
        let tmpl = civil_registry_template();
        let vr = validate(&tmpl, &gr.eav);
        assert!(vr.is_valid());
        store
            .commit(gr.event_kaki, gr.particle, gr.epoch, gr.eav)
            .unwrap();
    }

    #[test]
    fn epoch_zero_is_accepted() {
        let tid = TribeId::from_u16(0x0010);
        let gate = AdadGate::new(tid);
        let mut db = EnkiDb::new(tid);
        let mut store = PermanentStore::new(&mut db);
        commit_particle(&gate, &mut store, 0, ParticleState::Fuzzy);
        assert_eq!(store.stats().events_committed, 1);
    }

    #[test]
    fn epoch_max_u32_is_accepted() {
        let tid = TribeId::from_u16(0x0011);
        let gate = AdadGate::new(tid);
        let mut db = EnkiDb::new(tid);
        let mut store = PermanentStore::new(&mut db);
        commit_particle(&gate, &mut store, u32::MAX, ParticleState::Golden);
        assert_eq!(store.stats().events_committed, 1);
    }

    #[test]
    fn dead_particle_is_committed_and_projected() {
        let tid = TribeId::from_u16(0x0012);
        let gate = AdadGate::new(tid);
        let mut db = EnkiDb::new(tid);
        let p_id;
        {
            let record = ArrivalRecord {
                attrs: vec![(ATTR_STATE, encode_state(ParticleState::Dead).to_vec())],
                epoch: 99,
                role: KakiRole::Zikru,
            };
            let gr = gate.ingest(record).unwrap();
            let sec = check_sovereignty(tid, &gr.particle);
            assert!(sec.is_approved());
            let tmpl = civil_registry_template();
            let vr = validate(&tmpl, &gr.eav);
            assert!(vr.is_valid());
            p_id = gr.particle.clone();
            let mut store = PermanentStore::new(&mut db);
            store
                .commit(gr.event_kaki, gr.particle, gr.epoch, gr.eav)
                .unwrap();
        }
        let proj = db.project(&p_id);
        assert!(
            !proj.attributes.is_empty(),
            "Dead particle must still project to non-empty EAV"
        );
    }

    #[test]
    fn high_volume_100_particles() {
        let tid = TribeId::from_u16(0x0013);
        let gate = AdadGate::new(tid);
        let mut db = EnkiDb::new(tid);
        {
            let mut store = PermanentStore::new(&mut db);
            for ep in 0..100u32 {
                let st = match ep % 3 {
                    0 => ParticleState::Golden,
                    1 => ParticleState::Fuzzy,
                    _ => ParticleState::Dead,
                };
                commit_particle(&gate, &mut store, ep, st);
            }
            assert_eq!(store.stats().events_committed, 100);
        }
    }
}

#[cfg(test)]
mod malformed_input {
    use aaol::ast::Parser as AaolParser;
    use aaol::tokenize as aaol_lex;
    use heptascript::parse_query;

    #[test]
    fn heptascript_empty_source_returns_error_or_empty_plan() {
        match parse_query("") {
            Ok(plan) => assert_eq!(plan.r#where.len(), 0),
            Err(_) => { /* acceptable */ }
        }
    }

    #[test]
    fn heptascript_unknown_keyword_returns_error() {
        assert!(parse_query("DESTROY PARTICLE WHERE state = Golden").is_err());
    }

    #[test]
    fn heptascript_missing_value_returns_error() {
        assert!(parse_query("SELECT PARTICLE WHERE state =").is_err());
    }

    #[test]
    fn aaol_empty_source_parses_to_zero_statements() {
        let toks = aaol_lex("").expect("aaol_lex empty");
        let prog = AaolParser::new(toks).parse().expect("parser empty");
        assert_eq!(prog.statements.len(), 0);
    }

    #[test]
    fn aaol_comment_only_source_parses_cleanly() {
        let src = "# this is a comment\n# another comment\n";
        let toks = aaol_lex(src).expect("aaol_lex comment");
        let prog = AaolParser::new(toks).parse().expect("parser comment");
        assert_eq!(prog.statements.len(), 0);
    }

    #[test]
    fn aaol_unknown_token_returns_error() {
        assert!(aaol_lex("TRIBE @@ {}").is_err());
    }

    #[test]
    fn vgca_empty_eav_fails_all_required() {
        use template_library::operational_template;
        use vgca_validation::validate;
        let tmpl = operational_template();
        let vr = validate(&tmpl, &[]);
        assert!(
            !vr.is_valid(),
            "empty EAV must fail operational template validation"
        );
        assert_eq!(
            vr.missing_required.len(),
            3,
            "state+quality+freshness all required"
        );
    }
}

#[cfg(test)]
mod color_scoring {
    use score_engine::color_id::ColorRgb;

    #[test]
    fn golden_is_white() {
        let g = ColorRgb::GOLDEN;
        assert_eq!(g.r, 0xFF);
        assert_eq!(g.g, 0xFF);
        assert_eq!(g.b, 0xFF);
    }

    #[test]
    fn gray_has_mid_values() {
        let gr = ColorRgb::GRAY;
        assert_eq!(gr.r, 0x80);
        assert_eq!(gr.g, 0x80);
        assert_eq!(gr.b, 0x80);
    }

    #[test]
    fn drift_from_gray_for_golden_is_positive() {
        // Golden drifts from gray (gray is center, golden is bright)
        let drift = ColorRgb::GOLDEN.drift_from_gray();
        assert!(drift > 0.0, "GOLDEN must have positive drift from GRAY");
    }

    #[test]
    fn drift_from_gray_for_gray_is_zero() {
        let drift = ColorRgb::GRAY.drift_from_gray();
        assert!(drift < 1.0, "GRAY drift from itself must be near zero");
    }

    #[test]
    fn rgb_roundtrip() {
        let rgb = ColorRgb::new(0xAB, 0xCD, 0xEF);
        let b = rgb.to_bytes();
        let rgb2 = ColorRgb::from_bytes(b);
        assert_eq!(rgb, rgb2);
    }
}

#[cfg(test)]
mod eridu_resilience {
    use eridu_runtime::{EriduRuntime, Task, TaskResult};

    struct FailTask;
    impl Task for FailTask {
        fn name(&self) -> &str {
            "fail"
        }
        fn run(&mut self) -> TaskResult {
            TaskResult::Failed("intentional failure".to_string())
        }
    }

    struct RetryOnceTask {
        attempts: u32,
    }
    impl Task for RetryOnceTask {
        fn name(&self) -> &str {
            "retry_once"
        }
        fn run(&mut self) -> TaskResult {
            self.attempts += 1;
            if self.attempts < 2 {
                TaskResult::Retry
            } else {
                TaskResult::Ok
            }
        }
    }

    #[test]
    fn failed_tasks_increment_failed_counter() {
        let mut rt = EriduRuntime::new();
        rt.submit(Box::new(FailTask));
        rt.submit(Box::new(FailTask));
        rt.run_all();
        assert_eq!(rt.failed, 2);
        assert_eq!(rt.completed, 0);
    }

    #[test]
    fn retry_task_succeeds_on_second_attempt() {
        let mut rt = EriduRuntime::new();
        rt.submit(Box::new(RetryOnceTask { attempts: 0 }));
        rt.run_all();
        assert_eq!(rt.completed, 1);
        assert_eq!(rt.failed, 0);
    }

    #[test]
    fn empty_runtime_runs_zero_tasks() {
        let mut rt = EriduRuntime::new();
        let ran = rt.run_all();
        assert_eq!(ran, 0);
    }
}
