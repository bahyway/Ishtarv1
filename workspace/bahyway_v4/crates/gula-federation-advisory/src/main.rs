//! GulaFederationEngine — advisory face (SUSA boundary).
//! Read-only. Anonymous queries by design: no patient identity in
//! any request or log line (Najaf non-negotiables apply verbatim).
//! Every response is Ed25519-signed over its canonical JSON bytes.

use axum::{extract::{Path, State}, routing::get, Json, Router};
use base64::Engine as _;
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr, sync::Arc};

#[derive(Clone, Deserialize, Serialize)]
struct NodeStock {
    node_id: String,
    node_name: String,
    node_kind: String,        // hospital_pharmacy | public_pharmacy | wholesale
    lat: f64,
    lon: f64,
    open_24h: bool,
    reported_qty: u32,
    report_age_min: u32,      // staleness — shown honestly (epsilon)
}

#[derive(Serialize)]
struct Advisory {
    medicine: String,
    generated_utc: String,
    // epsilon doctrine: admitting uncertainty is itself transparency
    staleness_note: &'static str,
    anchors: Vec<NodeStock>,
}

#[derive(Serialize)]
struct SealedAdvisory {
    advisory: Advisory,
    seal_alg: &'static str,
    seal_b64: String,
    verify_key_b64: String,
}

struct AppState {
    key: SigningKey,
    federation: Vec<(String, Vec<NodeStock>)>, // medicine -> nodes
}

fn load_federation(path: &str) -> Vec<(String, Vec<NodeStock>)> {
    // Reads the synthetic federation read-model produced by PB-322.
    let raw = fs::read_to_string(path).unwrap_or_else(|_| "[]".into());
    serde_json::from_str(&raw).unwrap_or_default()
}

async fn advise(
    Path(medicine): Path<String>,
    State(st): State<Arc<AppState>>,
) -> Json<SealedAdvisory> {
    let needle = medicine.to_lowercase();
    let mut anchors: Vec<NodeStock> = st
        .federation
        .iter()
        .filter(|(m, _)| m.to_lowercase().contains(&needle))
        .flat_map(|(_, nodes)| nodes.clone())
        .filter(|n| n.reported_qty > 0)
        .collect();
    // Rank: fresh reports first, then quantity. Node character
    // (24h, kind) is left to the client-side advisory weighting.
    anchors.sort_by_key(|n| (n.report_age_min, u32::MAX - n.reported_qty));
    anchors.truncate(8);

    let advisory = Advisory {
        medicine,
        generated_utc: time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        staleness_note: "reported stock, not a reservation — call or go and ask",
        anchors,
    };
    let canonical = serde_json::to_vec(&advisory).expect("canonical json");
    let sig = st.key.sign(&canonical);
    Json(SealedAdvisory {
        advisory,
        seal_alg: "Ed25519",
        seal_b64: base64::engine::general_purpose::STANDARD.encode(sig.to_bytes()),
        verify_key_b64: base64::engine::general_purpose::STANDARD
            .encode(st.key.verifying_key().to_bytes()),
    })
}

#[tokio::main]
async fn main() {
    let data = std::env::var("GULA_FEDERATION_JSON")
        .unwrap_or_else(|_| "federation_readmodel.json".into());
    let key = SigningKey::generate(&mut rand::rngs::OsRng);
    let state = Arc::new(AppState { key, federation: load_federation(&data) });
    let app = Router::new()
        .route("/advisory/medicine/:name", get(advise))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive());
    let addr = SocketAddr::from(([0, 0, 0, 0], 7011));
    println!("GulaFederation advisory (read-only, advisory-only) on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
