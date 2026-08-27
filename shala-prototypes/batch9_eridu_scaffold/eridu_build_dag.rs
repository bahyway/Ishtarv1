// ============================================================================
// eridu_build_dag.rs — EriduScaffold build-DAG scheduler
//   Pure Rust · std only · ZERO external crates · sovereign
// ----------------------------------------------------------------------------
// PURPOSE (the honest-dogfooding law, made mechanical):
//   Documentation tasks form a Directed Acyclic Graph. Each task declares the
//   OntoGraph *patterns* it REQUIRES (must already exist) and the patterns it
//   PRODUCES. A task may run ONLY when every pattern it requires has been
//   produced by an already-completed task. Independent tasks run in parallel
//   (bounded to N workers). Application/service docs are additionally held by
//   an explicit STAGE GATE until every Foundational + Ontology task — the ones
//   that BUILD the OntoGraph — has completed and sealed.
//
//   => A downstream "app-doc generated from the OntoGraph" PHYSICALLY CANNOT
//      run before the OntoGraph it queries exists. Dogfooding is therefore
//      honest by construction: you can never document an application that does
//      not yet exist. No fabrication is possible.
//
// WHY NOT HYPERLEDGER: this is an ordering problem over a dependency graph you
//   already own (CrossTribe relations ARE the edges). It needs a topological
//   scheduler, not multi-party Byzantine consensus. You are a single sovereign
//   authority (CSR-08); Hyperledger's whole premise (no trusted authority) is
//   the thing you specifically do not want. The append-only tamper-evident
//   ledger a chain would give you, you already have: the zakāru journal +
//   Ed25519 seals + the Incorruptibility Chain (GL-OPS-001, L-8).
//
// SEAL NOTE: the log below is a std-only hash chain (FNV-1a) standing in for
//   the Incorruptibility Chain. In production each entry is Ed25519-signed at
//   the CSR-08 seal point (marked ▶ SEAL below) using ed25519-dalek.
//
// Algorithm pre-verified in Python before this Rust was written (waves,
// stage gate, honest-ordering proof, and chain integrity all confirmed).
//
// Build & run on the Fedora host:
//     rustc -O eridu_build_dag.rs -o eridu_dag && ./eridu_dag
// (or drop into a crate: `cargo new eridu-dag` and paste into src/main.rs)
// ============================================================================

use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// ---------------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stage {
    Foundational, // roots — populate the OntoGraph
    Ontology,     // build the OntoGraph proper
    Application,  // leaves — generated FROM the OntoGraph (gated)
}

impl Stage {
    fn label(&self) -> &'static str {
        match self {
            Stage::Foundational => "Foundational",
            Stage::Ontology => "Ontology",
            Stage::Application => "Application",
        }
    }
    /// The OntoGraph is "built" by everything that is not an Application doc.
    fn builds_ontograph(&self) -> bool {
        !matches!(self, Stage::Application)
    }
}

#[derive(Clone, Debug)]
struct Task {
    id: String,
    name: String,
    stage: Stage,
    requires: Vec<String>, // OntoGraph patterns that must exist first
    produces: Vec<String>, // patterns this task creates
}

impl Task {
    fn new(id: &str, name: &str, stage: Stage, req: &[&str], prod: &[&str]) -> Task {
        Task {
            id: id.into(),
            name: name.into(),
            stage,
            requires: req.iter().map(|s| s.to_string()).collect(),
            produces: prod.iter().map(|s| s.to_string()).collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// FNV-1a hash → 12 hex chars. Stand-in for the Incorruptibility Chain link.
// In production: replace with Ed25519 signature over the entry (CSR-08 seal).
fn chain_hash(seq: usize, task_id: &str, prev: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in format!("{seq}{task_id}{prev}").bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:012x}", h & 0xffff_ffff_ffff)
}

#[derive(Clone, Debug)]
struct SealEntry {
    seq: usize,
    task_id: String,
    stage: Stage,
    prev: String,
    hash: String,
}

// ---------------------------------------------------------------------------
// Kahn's algorithm: build task→task edges from pattern deps, detect cycles.
// Returns Ok(topo_order) or Err(remaining) if a cycle exists (not a DAG).
fn validate_acyclic(tasks: &[Task]) -> Result<Vec<String>, Vec<String>> {
    // producer[pattern] = task_id that produces it
    let mut producer: HashMap<String, String> = HashMap::new();
    for t in tasks {
        for p in &t.produces {
            producer.insert(p.clone(), t.id.clone());
        }
    }
    let mut edges: HashMap<String, HashSet<String>> = HashMap::new();
    let mut indeg: HashMap<String, usize> = HashMap::new();
    for t in tasks {
        edges.entry(t.id.clone()).or_default();
        indeg.entry(t.id.clone()).or_insert(0);
    }
    for t in tasks {
        for pat in &t.requires {
            if let Some(a) = producer.get(pat) {
                if edges.get_mut(a).unwrap().insert(t.id.clone()) {
                    *indeg.get_mut(&t.id).unwrap() += 1;
                }
            } else {
                // required pattern has no producer — an unsatisfiable input
                eprintln!("  ! task {} requires '{}' which no task produces", t.id, pat);
            }
        }
    }
    let mut queue: Vec<String> = indeg
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(k, _)| k.clone())
        .collect();
    queue.sort(); // deterministic
    let mut topo = Vec::new();
    let mut qi = 0;
    while qi < queue.len() {
        let n = queue[qi].clone();
        qi += 1;
        topo.push(n.clone());
        let mut outs: Vec<String> = edges[&n].iter().cloned().collect();
        outs.sort();
        for m in outs {
            let d = indeg.get_mut(&m).unwrap();
            *d -= 1;
            if *d == 0 {
                queue.push(m);
            }
        }
    }
    if topo.len() == tasks.len() {
        Ok(topo)
    } else {
        let done: HashSet<_> = topo.into_iter().collect();
        Err(tasks
            .iter()
            .map(|t| t.id.clone())
            .filter(|id| !done.contains(id))
            .collect())
    }
}

// ---------------------------------------------------------------------------
fn schedule(tasks: Vec<Task>, workers: usize) {
    println!("── EriduScaffold build-DAG · {} tasks · {} workers ──\n", tasks.len(), workers);

    // 1) validate the DAG is acyclic (else reject — cannot order a cycle)
    match validate_acyclic(&tasks) {
        Ok(order) => println!("DAG validated: ACYCLIC ✓  topo: {}\n", order.join(" → ")),
        Err(cycle) => {
            eprintln!("DAG REJECTED: cycle among {:?} — cannot guarantee ordering.", cycle);
            return;
        }
    }

    let by_id: HashMap<String, Task> =
        tasks.iter().map(|t| (t.id.clone(), t.clone())).collect();
    let onto_ids: Vec<String> = tasks
        .iter()
        .filter(|t| t.stage.builds_ontograph())
        .map(|t| t.id.clone())
        .collect();

    // worker pool: main feeds ready tasks; workers run them and report back
    let (tx_work, rx_work) = mpsc::channel::<Task>();
    let (tx_done, rx_done) = mpsc::channel::<Task>();
    let rx_work = Arc::new(Mutex::new(rx_work));
    let mut handles = Vec::new();
    for _ in 0..workers {
        let rx = Arc::clone(&rx_work);
        let td = tx_done.clone();
        handles.push(thread::spawn(move || loop {
            let job = { rx.lock().unwrap().recv() };
            match job {
                Ok(t) => {
                    // do the real work here (generate the doc / run the playbook).
                    thread::sleep(Duration::from_millis(60)); // simulated work
                    td.send(t).unwrap();
                }
                Err(_) => break, // channel closed → exit
            }
        }));
    }
    drop(tx_done); // only workers hold senders now

    let mut produced: HashSet<String> = HashSet::new();
    let mut started: HashSet<String> = HashSet::new();
    let mut completed: HashSet<String> = HashSet::new();
    let mut running = 0usize;
    let mut log: Vec<SealEntry> = Vec::new();
    let mut prev = String::from("GENESIS");
    let mut wave = 0usize;

    while completed.len() < tasks.len() {
        // gather all currently-ready tasks
        let mut ready: Vec<String> = Vec::new();
        for t in &tasks {
            if started.contains(&t.id) {
                continue;
            }
            // pattern readiness: every required pattern must already exist
            if !t.requires.iter().all(|r| produced.contains(r)) {
                continue;
            }
            // STAGE GATE: an Application doc waits until the whole OntoGraph is built
            if t.stage == Stage::Application
                && !onto_ids.iter().all(|o| completed.contains(o))
            {
                continue;
            }
            ready.push(t.id.clone());
        }
        ready.sort(); // deterministic dispatch

        if ready.is_empty() && running == 0 {
            eprintln!("DEADLOCK: no ready tasks and none running (invalid DAG?)");
            break;
        }

        if !ready.is_empty() {
            wave += 1;
            print!("wave {wave} (≤{workers} parallel): ");
            let mut names = Vec::new();
            for id in &ready {
                let t = by_id[id].clone();
                names.push(format!("{}·{}", id, t.name));
                started.insert(id.clone());
                tx_work.send(t).unwrap(); // bounded by N workers pulling
                running += 1;
            }
            println!("{}", names.join("  "));
        }

        // wait for at least one completion before re-scanning readiness
        if running > 0 {
            let done = rx_done.recv().unwrap();
            running -= 1;
            for p in &done.produces {
                produced.insert(p.clone());
            }
            completed.insert(done.id.clone());
            // ▶ SEAL: append-only, hash-chained (Ed25519-sign here in production)
            let seq = log.len() + 1;
            let h = chain_hash(seq, &done.id, &prev);
            log.push(SealEntry {
                seq,
                task_id: done.id.clone(),
                stage: done.stage,
                prev: prev.clone(),
                hash: h.clone(),
            });
            prev = h;
        }
    }

    drop(tx_work); // close work channel → workers exit
    for h in handles {
        let _ = h.join();
    }

    // ---- honest-ordering proof ----
    let seq_of: HashMap<&str, usize> =
        log.iter().map(|e| (e.task_id.as_str(), e.seq)).collect();
    let last_onto = onto_ids.iter().map(|o| seq_of[o.as_str()]).max().unwrap_or(0);
    let first_app = tasks
        .iter()
        .filter(|t| t.stage == Stage::Application)
        .map(|t| seq_of[t.id.as_str()])
        .min()
        .unwrap_or(usize::MAX);
    println!("\nPROOF · OntoGraph fully sealed by seq {last_onto} · first app-doc at seq {first_app}");
    println!(
        "        → {}",
        if last_onto < first_app {
            "HONEST by construction: no app-doc was generated before the OntoGraph existed."
        } else {
            "VIOLATION — ordering breached (should be impossible on a valid gate)."
        }
    );

    // ---- chain integrity (tamper-evidence) ----
    let mut ok = true;
    let mut p = String::from("GENESIS");
    for e in &log {
        if e.prev != p || chain_hash(e.seq, &e.task_id, &p) != e.hash {
            ok = false;
        }
        p = e.hash.clone();
    }
    println!(
        "CHAIN · {} entries · {}",
        log.len(),
        if ok { "VALID (tamper-evident)" } else { "BROKEN" }
    );
    println!("\nsealed append-only log:");
    for e in &log {
        println!(
            "  #{:<2} {:<3} [{:<12}] prev={} hash={}",
            e.seq,
            e.task_id,
            e.stage.label(),
            e.prev,
            e.hash
        );
    }
}

// ---------------------------------------------------------------------------
fn main() {
    // The documentation build DAG. Foundational + Ontology BUILD the OntoGraph;
    // Application docs are GENERATED FROM it and are gated until it exists.
    let tasks = vec![
        Task::new("F1", "define-tribes",       Stage::Foundational, &[],                                        &["tribe-model"]),
        Task::new("F2", "define-kaki-spec",    Stage::Foundational, &[],                                        &["kaki-spec"]),
        Task::new("F3", "define-law-catalog",  Stage::Foundational, &[],                                        &["law-catalog"]),
        Task::new("F4", "define-relations",    Stage::Foundational, &["tribe-model"],                           &["relation-model"]),
        Task::new("O1", "build-ontograph-core",Stage::Ontology,     &["tribe-model", "relation-model", "kaki-spec"], &["ontograph-core"]),
        Task::new("O2", "build-pattern-index", Stage::Ontology,     &["ontograph-core", "law-catalog"],         &["pattern-index"]),
        Task::new("A1", "gen-app-schema-doc",  Stage::Application,  &["ontograph-core", "pattern-index"],       &["app-schema-doc"]),
        Task::new("A2", "gen-service-doc",     Stage::Application,  &["ontograph-core"],                        &["service-doc"]),
        Task::new("A3", "gen-api-doc",         Stage::Application,  &["app-schema-doc"],                        &["api-doc"]),
    ];

    schedule(tasks, 3); // 3 bounded parallel workers, on demand / under control
}
