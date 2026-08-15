//! End-to-end integration tests — full pipeline from arrival to HeptaScript query.
//!
//! Each test exercises a complete vertical slice:
//!   AdadGate (ingest) → MusaruCheck (sovereignty) → VGCA (template validation)
//!   → PermanentStore (commit) → StoryEngine (project) → HeptaScript (query)
//!
//! No mocks. Everything runs in-memory using the same crates as production.

#[cfg(test)]
mod pipeline {
    use bahyway_core::{ParticleState, TribeId};
    use enkidb_engine::EnkiDb;
    use adad_gate::{AdadGate, ArrivalRecord};
    use enkidb_kaki::KakiRole;
    use musaru_security::check_sovereignty;
    use vgca_validation::validate;
    use permanent_storage::PermanentStore;
    use template_library::{civil_registry_template, operational_template, sensor_stream_template};
    use template_library::{ATTR_QUALITY, ATTR_FRESHNESS};
    use story_engine::projection::{encode_state, ATTR_STATE};
    use enkidb_query::query;

    fn tribe(n: u16) -> TribeId { TribeId::from_u16(n) }

    fn ingest_one(
        gate:    &AdadGate,
        store:   &mut PermanentStore,
        epoch:   u32,
        state:   ParticleState,
        extra:   Vec<(u32, Vec<u8>)>,
    ) -> bool {
        let mut attrs = vec![(ATTR_STATE, encode_state(state).to_vec())];
        attrs.extend(extra);

        let record = ArrivalRecord { attrs, epoch, role: KakiRole::Zikru };
        let gr = gate.ingest(record).expect("gate.ingest failed");

        let sec = check_sovereignty(gate.tribe_id(), &gr.particle);
        if !sec.is_approved() { return false; }

        let tmpl = civil_registry_template();
        let vr = validate(&tmpl, &gr.eav);
        if !vr.is_valid() { return false; }

        store.commit(gr.event_kaki, gr.particle, gr.epoch, gr.eav).is_ok()
    }

    // ── Test 1: single record round-trips through the full pipeline ──────────

    #[test]
    fn single_record_full_pipeline() {
        let tid   = tribe(0x0001);
        let gate  = AdadGate::new(tid);
        let mut db = EnkiDb::new(tid);

        {
            let mut store = PermanentStore::new(&mut db);
            let ok = ingest_one(&gate, &mut store, 1, ParticleState::Golden, vec![]);
            assert!(ok, "ingest_one should succeed for a valid Golden particle");
            assert_eq!(store.stats().events_committed, 1);
        }
    }

    // ── Test 2: multiple records accumulate in the journal ───────────────────

    #[test]
    fn multiple_records_accumulate() {
        let tid   = tribe(0x0002);
        let gate  = AdadGate::new(tid);
        let mut db = EnkiDb::new(tid);

        {
            let mut store = PermanentStore::new(&mut db);
            for (ep, st) in [(1, ParticleState::Golden), (2, ParticleState::Fuzzy), (3, ParticleState::Dead)] {
                assert!(ingest_one(&gate, &mut store, ep, st, vec![]), "epoch {ep} failed");
            }
            assert_eq!(store.stats().events_committed, 3);
        }
    }

    // ── Test 3: tribe mismatch is rejected by sovereignty check ─────────────

    #[test]
    fn sovereignty_rejects_tribe_mismatch() {
        let gate_tribe   = tribe(0x0010);
        let wrong_tribe  = tribe(0x0099);
        let gate         = AdadGate::new(gate_tribe);

        let record = ArrivalRecord {
            attrs: vec![(ATTR_STATE, encode_state(ParticleState::Golden).to_vec())],
            epoch: 1,
            role:  KakiRole::Zikru,
        };
        let gr = gate.ingest(record).expect("gate.ingest failed");

        let sec = check_sovereignty(wrong_tribe, &gr.particle);
        assert!(!sec.is_approved(), "sovereignty check must reject tribe mismatch");
    }

    // ── Test 4: VGCA rejects records missing required fields ─────────────────

    #[test]
    fn vgca_rejects_missing_required_fields() {
        use enkidb_journal::entry::EavTriple;
        // operational template requires state+quality+freshness
        let tmpl  = operational_template();
        // provide only state — quality and freshness are missing
        let eav = vec![EavTriple::new(ATTR_STATE, encode_state(ParticleState::Fuzzy).to_vec())];

        let vr = validate(&tmpl, &eav);
        assert!(!vr.is_valid(), "must reject when quality and freshness are absent");
        assert!(vr.missing_required.contains(&ATTR_QUALITY), "quality must be flagged");
        assert!(vr.missing_required.contains(&ATTR_FRESHNESS), "freshness must be flagged");
    }

    // ── Test 5: HeptaScript query returns committed particles ────────────────

    #[test]
    fn heptascript_query_finds_committed_particles() {
        let tid   = tribe(0x0003);
        let gate  = AdadGate::new(tid);
        let mut db = EnkiDb::new(tid);

        {
            let mut store = PermanentStore::new(&mut db);
            ingest_one(&gate, &mut store, 1, ParticleState::Golden, vec![]);
            ingest_one(&gate, &mut store, 2, ParticleState::Golden, vec![]);
            ingest_one(&gate, &mut store, 3, ParticleState::Dead,   vec![]);
        }

        let result = query(&db, "WHO Citizens.E\nWHY LANE = gold")
            .expect("HeptaScript query failed");

        assert!(result.evaluated > 0, "query must evaluate committed particles");
    }

    // ── Test 6: story-engine project reflects latest state ───────────────────

    #[test]
    fn story_engine_projects_latest_state() {
        let tid   = tribe(0x0004);
        let gate  = AdadGate::new(tid);
        let mut db = EnkiDb::new(tid);

        let p_id;

        {
            let record = ArrivalRecord {
                attrs: vec![(ATTR_STATE, encode_state(ParticleState::Golden).to_vec())],
                epoch: 1,
                role:  KakiRole::Zikru,
            };
            let gr = gate.ingest(record).unwrap();
            p_id = gr.particle.clone();

            let sec = check_sovereignty(tid, &gr.particle);
            assert!(sec.is_approved());

            let tmpl = civil_registry_template();
            let vr = validate(&tmpl, &gr.eav);
            assert!(vr.is_valid());

            let mut store = PermanentStore::new(&mut db);
            store.commit(gr.event_kaki, gr.particle, gr.epoch, gr.eav).unwrap();
        }

        let projected = db.project(&p_id);
        // Projection must contain at least the ATTR_STATE entry
        assert!(!projected.attributes.is_empty(), "project() must return non-empty EAV");
    }

    // ── Test 7: sensor.stream template allows optional color_rgb ─────────────

    #[test]
    fn sensor_stream_template_is_valid_without_color_rgb() {
        use enkidb_journal::entry::EavTriple;
        let tmpl = sensor_stream_template();

        let eav = vec![
            EavTriple::new(ATTR_STATE,     encode_state(ParticleState::Fuzzy).to_vec()),
            EavTriple::new(ATTR_FRESHNESS, vec![0xFF, 0xFF]),
        ];

        let vr = validate(&tmpl, &eav);
        assert!(vr.is_valid(), "sensor.stream must accept records without optional color_rgb");
    }
}

#[cfg(test)]
mod languages {
    use aaol::{tokenize as aaol_lex};
    use aaol::ast::Parser as AaolParser;
    use heptascript::{parse_query, tokenize as hepta_lex};

    // ── AAOL ─────────────────────────────────────────────────────────────────

    #[test]
    fn aaol_parses_tribe_declaration() {
        // Grammar: tribe Name { actor Name { role Name } }
        let src = r#"tribe Najaf { actor Registrar { role Zikru } }"#;
        let toks = aaol_lex(src).expect("aaol_lex failed");
        let prog = AaolParser::new(toks).parse().expect("AaolParser failed");
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn aaol_parses_event_rule() {
        // Grammar: when event "name" then emit snapshot;
        let src = r#"when event "particle.created" then emit snapshot;"#;
        let toks = aaol_lex(src).expect("aaol_lex failed");
        let prog = AaolParser::new(toks).parse().expect("AaolParser failed");
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn aaol_rejects_unknown_token() {
        let src = "@@@@";
        assert!(aaol_lex(src).is_err(), "aaol_lex must reject unknown chars");
    }

    // ── HeptaScript ──────────────────────────────────────────────────────────

    #[test]
    fn heptascript_parses_golden_query() {
        let plan = parse_query("WHO Citizens.E\nWHERE E[state] = \"Golden\"").expect("parse failed");
        assert_eq!(plan.r#where.len(), 1);
        assert!(plan.when.is_none());
    }
    #[test]
    fn heptascript_parses_time_travel_query() {
        let plan = parse_query("WHO Citizens.E\nWHEN AT EPOCH 42\nWHERE E[state] = \"Dead\"").expect("parse failed");
        assert!(plan.when.is_some());
    }
    #[test]
    fn heptascript_parses_compound_condition() {
        let plan = parse_query("WHO Citizens.E\nWHERE E[state] = \"Golden\"\nAND E[quality] > 0.8").expect("parse failed");
        assert_eq!(plan.r#where.len(), 2);
    }
    #[test]
    fn heptascript_tokenizer_is_case_insensitive() {
        let t1 = hepta_lex("WHO Citizens.E WHERE E[state] = \"Golden\"").expect("upper failed");
        let t2 = hepta_lex("who citizens.e where e[state] = \"golden\"").expect("lower failed");
        assert_eq!(t1.len(), t2.len(), "keyword case must not affect token count");
    }
}

#[cfg(test)]
mod runtime {
    use eridu_runtime::{Task, TaskResult, EriduRuntime};
    use eridu_supervisor::EriduSupervisor;
    use eridu_scheduler::ScheduledJob;

    struct CounterTask { pub count: u32 }
    impl Task for CounterTask {
        fn name(&self) -> &str { "counter" }
        fn run(&mut self) -> TaskResult { self.count += 1; TaskResult::Ok }
    }

    #[test]
    fn runtime_runs_all_tasks() {
        let mut rt = EriduRuntime::new();
        rt.submit(Box::new(CounterTask { count: 0 }));
        rt.submit(Box::new(CounterTask { count: 0 }));
        rt.submit(Box::new(CounterTask { count: 0 }));
        let ran = rt.run_all();
        assert_eq!(ran, 3);
        assert_eq!(rt.completed, 3);
        assert_eq!(rt.failed, 0);
    }

    #[test]
    fn supervisor_fires_scheduled_jobs() {
        let mut sup = EriduSupervisor::new();
        sup.start();
        sup.register_job(ScheduledJob::new("heartbeat", 5));

        // Advance 1 tick at a time; interval=5 means:
        //   tick 1: first fire (last_run==0), last_run=1
        //   ticks 2-5: current-1 < 5, no fire
        //   tick 6: current=6, 6-1=5 >= 5, fires again

        let due = sup.tick(1, |_| Box::new(CounterTask { count: 0 }) as Box<dyn Task>);
        assert!(!due.is_empty(), "heartbeat must fire on first tick");
        sup.run_pending();

        // Ticks 2-5: should NOT fire (4 advances of 1)
        for _ in 0..4 {
            let due2 = sup.tick(1, |_| Box::new(CounterTask { count: 0 }) as Box<dyn Task>);
            assert!(due2.is_empty(), "heartbeat must not fire before interval elapses");
        }

        // Tick 6 (1 more advance): fires again (current=6, 6-1=5 >= interval=5)
        let due6 = sup.tick(1, |_| Box::new(CounterTask { count: 0 }) as Box<dyn Task>);
        assert!(!due6.is_empty(), "heartbeat must fire again after interval ticks");
    }

    #[test]
    fn supervisor_health_is_healthy_after_clean_run() {
        use eridu_supervisor::HealthStatus;
        let mut sup = EriduSupervisor::new();
        sup.start();
        sup.register_job(ScheduledJob::new("job", 1));
        sup.tick(1, |_| Box::new(CounterTask { count: 0 }) as Box<dyn Task>);
        sup.run_pending();
        assert_eq!(sup.health(), HealthStatus::Healthy);
    }
}

// ── Five-tier EnkiDB pipeline integration tests ──────────────────────────────
// Covers the full flow:
//   ZIP arrives → SdbPipeline stages particles → SchedulerLoop tick →
//   ValidationSweep → valid→ODB, malware→QDB → DiagnosisEngine scans ODB
//
// Sweep interval is set to 10 ticks (not 900) so tests run instantly.
#[cfg(test)]
mod five_tier_pipeline {
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiRole;
    use enkisdb::sdb_store::{SdbStatus, StagedParticle};
    use enkidw::build_store_zip;
    use eridu_runtime::SchedulerLoop;
    use diagnosis_engine::DiagnosisEngine;
    use std::fs;
    use std::path::PathBuf;

    fn tid() -> TribeId { TribeId::from_u16(0x0001) }

    fn tmp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("e2e_5tier_{}", tag));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn make_loop() -> SchedulerLoop {
        SchedulerLoop::with_interval(tid(), 64, 10)
    }

    fn stage_particle(lp: &mut SchedulerLoop, malware: bool) {
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(
            lp.minter.identity(KakiRole::Zikru)
        ).unwrap();
        lp.sdb.stage(StagedParticle {
            kaki_bytes:   *ik.bytes(),
            tribe_id:     tid().as_u16(),
            epoch:        1,
            eav:          Vec::new(),
            color_rgb:    if malware { [255, 0, 0] } else { [80, 200, 120] },
            status:       SdbStatus::Pending,
            arrived_tick: 0,
            malware_flag: malware,
        });
    }

    // ── Test 1: clean batch → all land in ODB ─────────────────────────────────

    #[test]
    fn clean_batch_promotes_all_to_odb() {
        let mut lp = make_loop();
        stage_particle(&mut lp, false);
        stage_particle(&mut lp, false);
        stage_particle(&mut lp, false);

        let outcome = lp.tick(1); // sweep fires on first tick (last_run=0)

        assert!(outcome.sweep_ran);
        assert_eq!(outcome.promoted,    3);
        assert_eq!(outcome.quarantined, 0);
        assert_eq!(lp.odb.stats().active_count, 3);
        assert_eq!(lp.qdb.len(), 0);
    }

    // ── Test 2: mixed batch → split between ODB and QDB ──────────────────────

    #[test]
    fn mixed_batch_splits_correctly() {
        let mut lp = make_loop();
        stage_particle(&mut lp, false); // valid
        stage_particle(&mut lp, false); // valid
        stage_particle(&mut lp, true);  // malware → BlackBox → Storage Sector

        lp.tick(1);

        assert_eq!(lp.odb.stats().active_count, 2);
        assert_eq!(lp.qdb.len(),                0);
        assert_eq!(lp.storage_sector.len(),     1);
    }

    // ── Test 3: ZIP via SdbPipeline → staged → sweep → ODB ──────────────────

    #[test]
    fn zip_pipeline_full_flow() {
        let base   = tmp_dir("zip_flow");
        let mut lp = make_loop();

        // Build a clean ZIP with a TSV inside
        let tsv = b"name\tepoch\tstate\nAli_Karim\t1\tGolden\nFatima\t2\tFuzzy\n";
        let zip = build_store_zip("records.tsv", tsv);
        fs::write(base.join("landing").join("batch.zip"), &zip).unwrap_or_else(|_| {
            fs::create_dir_all(base.join("landing")).unwrap();
            fs::write(base.join("landing").join("batch.zip"), &zip).unwrap();
        });

        // Run the SdbPipeline to stage particles from the ZIP
        let mut pipe = enkisdb::SdbPipeline::open(&base.join("landing"), tid()).unwrap();
        pipe.run_once(&mut lp.sdb, &mut lp.journal);

        assert_eq!(pipe.stats().records_staged, 2);
        assert_eq!(lp.sdb.stats().total_staged, 2);

        // Now tick to trigger the sweep
        let outcome = lp.tick(1);
        assert!(outcome.sweep_ran);
        assert_eq!(lp.odb.stats().active_count, 2);
        assert_eq!(lp.qdb.len(), 0);

        let _ = fs::remove_dir_all(&base);
    }

    // ── Test 4: malware ZIP → malware flag → BlackBox seals into Storage Sector ──

    #[test]
    fn malware_zip_routed_to_storage_sector() {
        let base   = tmp_dir("malware_zip");
        let mut lp = make_loop();

        // Embed EICAR signature in ZIP payload
        let tsv = b"name\nSuspect\n";
        let mut zip = build_store_zip("data.tsv", tsv);
        zip.extend_from_slice(b"EICAR-STANDARD-ANTIVIRUS-TEST-FILE");
        fs::create_dir_all(base.join("landing")).unwrap();
        fs::write(base.join("landing").join("malware.zip"), &zip).unwrap();

        let mut pipe = enkisdb::SdbPipeline::open(&base.join("landing"), tid()).unwrap();
        pipe.run_once(&mut lp.sdb, &mut lp.journal);

        assert_eq!(pipe.stats().malware_hits, 1);
        assert!(lp.sdb.all()[0].malware_flag);

        lp.tick(1);

        assert_eq!(lp.odb.stats().active_count, 0);
        assert_eq!(lp.qdb.len(), 0);
        assert_eq!(lp.storage_sector.len(), 1);

        let _ = fs::remove_dir_all(&base);
    }

    // ── Test 5: second sweep does not re-ingest already-promoted particles ────

    #[test]
    fn idempotent_drain_across_multiple_sweeps() {
        let mut lp = make_loop();
        stage_particle(&mut lp, false);

        lp.tick(1);  // first sweep: 1 promoted, 1 drained to ODB
        let odb_after_first = lp.odb.stats().active_count;

        // Stage one more particle before second sweep
        stage_particle(&mut lp, false);
        lp.tick(10); // second sweep fires (interval=10)

        assert_eq!(lp.odb.stats().active_count, odb_after_first + 1,
            "second sweep must only promote the new particle, not re-ingest the first");
    }

    // ── Test 6: Journal contains all expected EventCauses ────────────────────

    #[test]
    fn journal_audit_trail_complete() {
        let mut lp = make_loop();
        stage_particle(&mut lp, false); // valid
        stage_particle(&mut lp, true);  // malware

        lp.tick(1);

        // Expect: 2× SdbValidation(Pass/Fail) from sweep + 1× SdbValidationPass from ODB ingest + 1× StorageSectorMove
        let count = lp.journal.entry_count();
        assert!(count >= 3, "expected at least 3 journal entries, got {count}");
    }

    // ── Test 7: DiagnosisEngine scans ODB particles in Journal ───────────────

    #[test]
    fn diagnosis_engine_detects_drift_after_promotion() {
        let mut lp  = make_loop();

        // Stage a particle with color very far from the tribe root
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(
            lp.minter.identity(KakiRole::Zikru)
        ).unwrap();
        // color_rgb = [0,0,0] vs root = [128,200,255] → large drift → Critical
        lp.sdb.stage(StagedParticle {
            kaki_bytes:   *ik.bytes(),
            tribe_id:     tid().as_u16(),
            epoch:        1,
            eav:          Vec::new(),
            color_rgb:    [0, 0, 0],
            status:       SdbStatus::Pending,
            arrived_tick: 0,
            malware_flag: false,
        });

        lp.tick(1); // sweep → particle in ODB

        // Run DiagnosisEngine
        let mut eng = DiagnosisEngine::new();
        eng.set_tribe_root_color(tid().as_u16(), [128, 200, 255]);
        let res = eng.run(&mut lp.journal, &lp.minter, tid(), 0);

        assert!(res.critical_events > 0,
            "expected at least one critical event for highly drifted particle");
    }

    // ── Test 8: fuzzy (tribe-mismatch, non-malware) particle → BlackBox →
    //    EnkiQDB (never Storage Sector) → Data Steward review → resolve
    //    clean → requeued into EnkiSDB ────────────────────────────────────

    #[test]
    fn fuzzy_particle_routes_to_qdb_and_steward_resolves_clean() {
        use data_steward_station::QuarantineReviewQueue;

        let mut lp = make_loop();

        // A tribe-mismatched, non-malware particle: ValidationSweep quarantines
        // it (sovereignty check fails) but malware_flag stays false, so this is
        // the "fuzzy/unknown" case, not a confirmed-harmful one.
        let foreign_minter = enkidb_kaki::KakiMinter::new(TribeId::from_u16(0x0002));
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(
            foreign_minter.identity(KakiRole::Zikru)
        ).unwrap();
        lp.sdb.stage(StagedParticle {
            kaki_bytes:   *ik.bytes(),
            tribe_id:     0x0002,
            epoch:        1,
            eav:          Vec::new(),
            color_rgb:    [90, 90, 90],
            status:       SdbStatus::Pending,
            arrived_tick: 0,
            malware_flag: false,
        });

        let outcome = lp.tick(1);
        assert_eq!(outcome.quarantined, 1);
        assert_eq!(outcome.routed_to_qdb, 1);
        assert_eq!(outcome.routed_to_storage_sector, 0);
        assert_eq!(lp.qdb.len(), 1);
        assert_eq!(lp.storage_sector.len(), 0);

        // Data Steward pulls the fuzzy backlog from EnkiQDB.
        let mut queue = QuarantineReviewQueue::new();
        assert_eq!(queue.pull_from_qdb(&lp.qdb), 1);

        // Steward resolves it clean: a fresh Pending particle re-enters EnkiSDB.
        let tick    = lp.current_tick();
        let new_idx = queue
            .resolve_clean(0, &mut lp.sdb, &mut lp.journal, &lp.minter, tick)
            .expect("resolve_clean should requeue into SDB");
        assert!(queue.is_empty());
        assert_eq!(lp.sdb.get(new_idx).unwrap().status, SdbStatus::Pending);
        assert!(!lp.sdb.get(new_idx).unwrap().malware_flag);

        // EnkiQDB is append-only: the original record is untouched, not removed.
        assert_eq!(lp.qdb.len(), 1);
    }

    // ── Test 9: Data Steward can instead confirm a fuzzy case as harmful,
    //    sealing it into the Storage Sector directly from the review queue ──

    #[test]
    fn steward_can_confirm_fuzzy_case_as_harmful() {
        use data_steward_station::QuarantineReviewQueue;

        let mut lp = make_loop();

        let foreign_minter = enkidb_kaki::KakiMinter::new(TribeId::from_u16(0x0002));
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(
            foreign_minter.identity(KakiRole::Zikru)
        ).unwrap();
        lp.sdb.stage(StagedParticle {
            kaki_bytes:   *ik.bytes(),
            tribe_id:     0x0002,
            epoch:        1,
            eav:          Vec::new(),
            color_rgb:    [90, 90, 90],
            status:       SdbStatus::Pending,
            arrived_tick: 0,
            malware_flag: false,
        });

        lp.tick(1);
        assert_eq!(lp.qdb.len(), 1);

        let mut queue = QuarantineReviewQueue::new();
        queue.pull_from_qdb(&lp.qdb);

        let tick = lp.current_tick();
        let sealed_idx = queue
            .resolve_confirmed_harmful(0, &mut lp.storage_sector, &mut lp.journal, &lp.minter, tick)
            .expect("resolve_confirmed_harmful should seal into Storage Sector");

        assert!(queue.is_empty());
        assert_eq!(lp.storage_sector.len(), 1);
        assert_eq!(lp.storage_sector.all()[sealed_idx].sealed_tick, tick);
        // EnkiQDB's original record is still there — append-only audit trail.
        assert_eq!(lp.qdb.len(), 1);
    }
}

