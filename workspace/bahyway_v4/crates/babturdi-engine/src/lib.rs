//! BabTurdiEngine (GL-NRG-001-A2): cluster a client's REFUSED stream
//! by meaning, attribute defense cost with tau, gate the reward tier.
//! NEVER links a Non-Particle to an inner Particle (No-Contamination).
#![forbid(unsafe_code)]

/// A refused Record on the SUSA ledger — NO KAKI, by construction.
#[derive(Clone)]
pub struct Refused {
    pub client: String,
    pub endpoint: String,
    pub vec: [f64; 4],     // semantic embedding of the refusal reason
    pub on_hot_spot: bool, // did it land on the strained patch?
    pub severity: f64,     // 0 formatting .. 1 malware payload
}

/// A cost-attribution result. NOTE: there is NO field for a Particle
/// identity anywhere in this type — the No-Contamination boundary is
/// enforced by the type system (L61).
#[derive(Clone, Debug)]
pub struct CostSignature {
    pub client: String,
    pub n: usize,
    pub hot_share: f64,     // fraction on the strained spot
    pub malware_share: f64, // fraction high-severity
    pub cost: f64,          // attributed defense cost
    pub tau: f64,           // uncertainty (never a bare number, L62)
    pub monthly: Vec<f64>,  // per-window costs, NEVER averaged (L64)
}

fn cosine(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let (mut d, mut na, mut nb) = (0.0, 0.0, 0.0);
    for i in 0..4 {
        d += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    d / (na.sqrt() * nb.sqrt() + 1e-9)
}

/// Cost model: each refused record costs base + severity weight +
/// hot-spot surcharge (defending the strained patch is dearer).
fn record_cost(r: &Refused) -> f64 {
    1.0 + 2.5 * r.severity + if r.on_hot_spot { 1.8 } else { 0.0 }
}

/// Attribute cost to ONE client from the outer ledger only.
pub fn attribute(client: &str, log: &[Refused], windows: usize) -> CostSignature {
    let mine: Vec<&Refused> = log.iter().filter(|r| r.client == client).collect();
    let n = mine.len();
    let hot = mine.iter().filter(|r| r.on_hot_spot).count() as f64 / n.max(1) as f64;
    let mal = mine.iter().filter(|r| r.severity > 0.6).count() as f64 / n.max(1) as f64;
    let costs: Vec<f64> = mine.iter().map(|r| record_cost(r)).collect();
    let total: f64 = costs.iter().sum();
    let mean = total / n.max(1) as f64;
    let var = costs.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / n.max(1) as f64;
    let tau = (var / n.max(1) as f64).sqrt();
    // per-window costs (never averaged into one blaming number, L64)
    let mut monthly = vec![0.0; windows];
    for (k, r) in mine.iter().enumerate() {
        monthly[k % windows] += record_cost(r);
    }
    CostSignature {
        client: client.into(),
        n,
        hot_share: hot,
        malware_share: mal,
        cost: total,
        tau,
        monthly,
    }
}

/// Reward gate: the low-cost tier is EARNED by measured low cost,
/// never granted by default. Above the cohort median -> not earned (L63).
#[derive(PartialEq, Debug)]
pub enum Reward {
    LowCostEarned,
    NotEarned,
}
pub fn reward_tier(sig: &CostSignature, cohort_median: f64) -> Reward {
    if sig.cost <= cohort_median {
        Reward::LowCostEarned
    } else {
        Reward::NotEarned
    }
}

/// Semantic clustering of a client's stream by meaning (for the WHY).
pub fn cluster_by_meaning(client: &str, log: &[Refused], centroids: &[[f64; 4]]) -> Vec<usize> {
    let mut counts = vec![0usize; centroids.len()];
    for r in log.iter().filter(|r| r.client == client) {
        let mut best = 0;
        let mut bs = -2.0;
        for (ci, c) in centroids.iter().enumerate() {
            let s = cosine(&r.vec, c);
            if s > bs {
                bs = s;
                best = ci;
            }
        }
        counts[best] += 1;
    }
    counts
}

pub fn cohort_median(sigs: &[CostSignature]) -> f64 {
    let mut c: Vec<f64> = sigs.iter().map(|s| s.cost).collect();
    c.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = c.len();
    if n == 0 {
        0.0
    } else if n % 2 == 1 {
        c[n / 2]
    } else {
        // even count: true median is the mean of the two middle values,
        // not the upper one — picking the upper value collapses to the
        // max whenever the cohort has exactly two members, letting the
        // costlier client's own cost define the bar it must clear (L63).
        (c[n / 2 - 1] + c[n / 2]) / 2.0
    }
}
