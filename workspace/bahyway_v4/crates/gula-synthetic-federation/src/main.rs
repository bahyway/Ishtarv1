//! Synthetic province generator. Everything is a particle:
//! nodes, products, batches, and price observations. Quality,
//! state, and prices live in EAV-style attributes ONLY — the
//! KAKI byte layout is never abused for them (locked law).

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use std::fs;

const SEED: u64 = 0x4755_4C41; // "GULA"

#[derive(Serialize)]
struct Node { node_id: String, node_name: String, node_kind: String,
              lat: f64, lon: f64, open_24h: bool }

#[derive(Serialize)]
struct Batch {
    batch_id: String, product: String, atc: String, node_id: String,
    qty: u32, mfg_day: i64, expiry_day: i64,     // days from today
    assay_pct: f64,                              // API vs spec, 100 = exact
    storage: String,                             // ambient | cold_2_8
    accept_state: String,                        // accepted | rejected | held
    price_official: f64, price_commercial: f64, price_black: f64,
    cold_recovery_slope: Option<f64>,            // Asakku-style, cold chain only
    report_age_min: u32,
}

fn main() {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let districts = ["Karkh","Rusafa","Kadhimiya","Adhamiyah","Sadr City",
                     "Mansour","Karrada","Dora","Shaab","New Baghdad"];
    let mut nodes = Vec::new();
    for i in 0..400 {
        nodes.push(Node{ node_id: format!("PH-{:03}", i),
            node_name: format!("{} Pharmacy {}", districts[i % districts.len()], i),
            node_kind: "public_pharmacy".into(),
            lat: 33.20 + rng.gen::<f64>() * 0.35,
            lon: 44.25 + rng.gen::<f64>() * 0.35,
            open_24h: rng.gen_bool(0.08) });
    }
    for i in 0..40 {
        nodes.push(Node{ node_id: format!("HP-{:02}", i),
            node_name: format!("Gov Hospital Pharmacy {}", i),
            node_kind: "hospital_pharmacy".into(),
            lat: 33.22 + rng.gen::<f64>() * 0.30,
            lon: 44.28 + rng.gen::<f64>() * 0.30,
            open_24h: true });
    }
    for i in 0..12 {
        nodes.push(Node{ node_id: format!("WH-{:02}", i),
            node_name: format!("Provincial Warehouse {}", i),
            node_kind: "wholesale".into(),
            lat: 33.25 + rng.gen::<f64>() * 0.25,
            lon: 44.30 + rng.gen::<f64>() * 0.25,
            open_24h: false });
    }

    let inn = ["Amoxicillin","Ceftriaxone","Metformin","Insulin glargine",
               "Amlodipine","Omeprazole","Paracetamol","Salbutamol",
               "Enoxaparin","Azithromycin","Losartan","Atorvastatin",
               "ORS","Diazepam","Furosemide","Levothyroxine"];
    let forms = ["250mg tab","500mg tab","1g inj","100IU/ml vial",
                 "5mg tab","20mg cap","syrup 125mg/5ml","inhaler 100mcg"];

    let mut batches = Vec::new();
    let mut products = 0usize;
    'outer: for a in inn { for f in forms {
        for strength_var in 0..63 {           // ~16*8*63 ≈ 8064 products
            products += 1;
            if products > 8000 { break 'outer; }
            let product = format!("{a} {f} v{strength_var}");
            let n_batches = 1 + (rng.gen::<u32>() % 3);
            for b in 0..n_batches {
                let node = &nodes[rng.gen_range(0..nodes.len())];
                let cold = a == "Insulin glargine" || a == "Enoxaparin"
                           || f.contains("inj") || f.contains("vial");
                let expiry = rng.gen_range(-60..720);
                let assay: f64 = 100.0 + rng.gen_range(-8.0..4.0);
                // 2026-08-23, found live: E0689 "can't call method `round`
                // on ambiguous numeric type `{float}`" -- po and spread had
                // no type annotation, and unlike integers, float literals
                // have no default fallback type in Rust. Multiplying two
                // still-ambiguous values together (po * spread) never
                // resolved either one. Explicit f64 fixes it.
                let po: f64 = rng.gen_range(0.5..40.0);
                let spread: f64 = rng.gen_range(1.0..3.2);
                batches.push(Batch{
                    batch_id: format!("BGH-{:04}-{:04}", products, b),
                    product: product.clone(),
                    atc: format!("{}{:02}", &a[..1], strength_var % 20),
                    node_id: node.node_id.clone(),
                    qty: rng.gen_range(0..500),
                    mfg_day: expiry - 730, expiry_day: expiry,
                    assay_pct: (assay * 10.0).round() / 10.0,
                    storage: if cold {"cold_2_8".into()} else {"ambient".into()},
                    accept_state: if assay < 90.0 {"rejected".into()}
                                  else if rng.gen_bool(0.03) {"held".into()}
                                  else {"accepted".into()},
                    price_official: (po * 100.0).round() / 100.0,
                    price_commercial: (po * rng.gen_range(1.0..1.6) * 100.0).round() / 100.0,
                    price_black: (po * spread * 100.0).round() / 100.0,
                    cold_recovery_slope: cold.then(|| (rng.gen_range(-0.30..0.05f64) * 1000.0).round() / 1000.0),
                    report_age_min: rng.gen_range(1..2880),
                });
            }
        }
    }}

    fs::write("nodes.json", serde_json::to_string_pretty(&nodes).unwrap()).unwrap();
    fs::write("batches.json", serde_json::to_string_pretty(&batches).unwrap()).unwrap();

    // Advisory read-model for PB-321: medicine -> stocked anchors
    use std::collections::BTreeMap;
    let mut rm: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    for b in &batches {
        if b.qty == 0 || b.accept_state != "accepted" || b.expiry_day <= 0 { continue; }
        let node = nodes.iter().find(|n| n.node_id == b.node_id).unwrap();
        rm.entry(b.product.clone()).or_default().push(serde_json::json!({
            "node_id": node.node_id, "node_name": node.node_name,
            "node_kind": node.node_kind, "lat": node.lat, "lon": node.lon,
            "open_24h": node.open_24h, "reported_qty": b.qty,
            "report_age_min": b.report_age_min }));
    }
    let rm_vec: Vec<(String, &Vec<serde_json::Value>)> =
        rm.iter().map(|(k, v)| (k.clone(), v)).collect();
    fs::write("federation_readmodel.json",
              serde_json::to_string(&rm_vec).unwrap()).unwrap();
    println!("nodes={} batches={} products~{}", nodes.len(), batches.len(), products.min(8000));
}
