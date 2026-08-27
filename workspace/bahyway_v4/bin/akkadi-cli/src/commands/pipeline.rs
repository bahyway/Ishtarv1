use crate::output::format::OutputFormat;

struct Station {
    name: &'static str,
    throughput: &'static str,
    latency_ms: f64,
}

const LANES: [(&str, &[Station]); 4] = [
    (
        "Lane A — Ingest",
        &[
            Station {
                name: "VaultStore",
                throughput: "12k/s",
                latency_ms: 0.08,
            },
            Station {
                name: "KAKI Issuance",
                throughput: "12k/s",
                latency_ms: 0.04,
            },
            Station {
                name: "FuzzyEngine 8D",
                throughput: "8k/s",
                latency_ms: 0.31,
            },
            Station {
                name: "CompareEngine",
                throughput: "8k/s",
                latency_ms: 0.22,
            },
            Station {
                name: "Final Score",
                throughput: "8k/s",
                latency_ms: 0.12,
            },
            Station {
                name: "Golden Record",
                throughput: "2.8k/s",
                latency_ms: 0.18,
            },
            Station {
                name: "Dead Archive",
                throughput: "4.7k/s",
                latency_ms: 0.09,
            },
            Station {
                name: "Security Seal",
                throughput: "12k/s",
                latency_ms: 0.03,
            },
        ],
    ),
    (
        "Lane B — Validation",
        &[
            Station {
                name: "VaultStore",
                throughput: "10k/s",
                latency_ms: 0.08,
            },
            Station {
                name: "KAKI Issuance",
                throughput: "10k/s",
                latency_ms: 0.04,
            },
            Station {
                name: "FuzzyEngine 8D",
                throughput: "7k/s",
                latency_ms: 0.31,
            },
            Station {
                name: "CompareEngine",
                throughput: "7k/s",
                latency_ms: 0.22,
            },
            Station {
                name: "Final Score",
                throughput: "7k/s",
                latency_ms: 0.12,
            },
            Station {
                name: "Golden Record",
                throughput: "2.4k/s",
                latency_ms: 0.18,
            },
            Station {
                name: "Dead Archive",
                throughput: "4.1k/s",
                latency_ms: 0.09,
            },
            Station {
                name: "Security Seal",
                throughput: "10k/s",
                latency_ms: 0.03,
            },
        ],
    ),
    (
        "Lane C — CrossTribe",
        &[
            Station {
                name: "VaultStore",
                throughput: "6k/s",
                latency_ms: 0.08,
            },
            Station {
                name: "KAKI Issuance",
                throughput: "6k/s",
                latency_ms: 0.04,
            },
            Station {
                name: "FuzzyEngine 8D",
                throughput: "4k/s",
                latency_ms: 0.55,
            },
            Station {
                name: "CompareEngine",
                throughput: "4k/s",
                latency_ms: 0.44,
            },
            Station {
                name: "Final Score",
                throughput: "4k/s",
                latency_ms: 0.18,
            },
            Station {
                name: "Golden Record",
                throughput: "1.4k/s",
                latency_ms: 0.24,
            },
            Station {
                name: "Dead Archive",
                throughput: "2.3k/s",
                latency_ms: 0.12,
            },
            Station {
                name: "Security Seal",
                throughput: "6k/s",
                latency_ms: 0.03,
            },
        ],
    ),
    (
        "Lane D — Audit",
        &[
            Station {
                name: "VaultStore",
                throughput: "20k/s",
                latency_ms: 0.06,
            },
            Station {
                name: "KAKI Issuance",
                throughput: "20k/s",
                latency_ms: 0.03,
            },
            Station {
                name: "FuzzyEngine 8D",
                throughput: "15k/s",
                latency_ms: 0.25,
            },
            Station {
                name: "CompareEngine",
                throughput: "15k/s",
                latency_ms: 0.18,
            },
            Station {
                name: "Final Score",
                throughput: "15k/s",
                latency_ms: 0.09,
            },
            Station {
                name: "Golden Record",
                throughput: "5k/s",
                latency_ms: 0.14,
            },
            Station {
                name: "Dead Archive",
                throughput: "8k/s",
                latency_ms: 0.07,
            },
            Station {
                name: "Security Seal",
                throughput: "20k/s",
                latency_ms: 0.02,
            },
        ],
    ),
];

pub fn pipeline_stats(format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let mut out = serde_json::Map::new();
            for (lane, stations) in &LANES {
                let arr: Vec<serde_json::Value> = stations
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "station":      s.name,
                            "throughput":   s.throughput,
                            "latency_ms":   s.latency_ms,
                        })
                    })
                    .collect();
                out.insert(lane.to_string(), serde_json::Value::Array(arr));
            }
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        OutputFormat::Cuneiform => {
            println!("𒀭𒆳𒁺 BeeMDM Pipeline — 4 lanes × 8 stations (ADR-004)");
            for (lane, stations) in &LANES {
                println!("\n  𒌋𒌋 {lane}");
                for (i, s) in stations.iter().enumerate() {
                    println!(
                        "    {}. {} — {} | {:.2}ms",
                        i + 1,
                        s.name,
                        s.throughput,
                        s.latency_ms
                    );
                }
            }
        }
        OutputFormat::Table => {
            println!("BeeMDM Pipeline — 4 lanes × 8 stations (ADR-004)\n");
            for (lane, stations) in &LANES {
                println!("  {lane}");
                println!("  {:<22} {:>10} {:>10}", "STATION", "THROUGHPUT", "LATENCY");
                println!("  {}", "─".repeat(46));
                for (i, s) in stations.iter().enumerate() {
                    println!(
                        "  {:2}. {:<18} {:>10} {:>9.2}ms",
                        i + 1,
                        s.name,
                        s.throughput,
                        s.latency_ms
                    );
                }
                println!();
            }
        }
    }
}
