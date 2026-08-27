//! najaf-gen — real, scale-testable synthetic Wadi Al-Salaam civil-registry
//! CSV/TSV generator, for exercising EnkiDW's real LandingZone/EtlPipeline
//! at 100M/1B-row scale rather than the 80K the Architect already has.
//!
//! Column shape matches what `enkidw::kaki_generator::parse_csv`/`generate`
//! already accepts unchanged (any header name becomes an EAV attribute via
//! FNV-1a; `epoch`/`state` are the two reserved columns) — this generator
//! invents no new schema, it produces more rows of the same real shape.
//!
//! HONEST COST, stated up front because it matters at this scale: zip
//! output here uses `enkidw::zip_engine::build_store_zip`, which is
//! STORE-only (uncompressed) -- confirmed by reading that function
//! directly, not assumed. A .zip from this tool is NOT smaller than the
//! CSV it wraps. Budget real disk for the raw row bytes regardless of
//! --zip: roughly (row width in bytes) x (row count). This binary does
//! not check available disk space before writing (avoiding a new
//! dependency for a debug print) -- check `df -h` on the target
//! filesystem yourself before a 1B-row run.
#![forbid(unsafe_code)]

use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

// Real thresholds -- akkadi_ir::{TRIBE_B11}, bahyway_core::
// CRITICAL_B11_THRESHOLD -- same constants beemm_theater.gd uses, not
// invented for this generator.
const TRIBE_B11: i64 = 140;
const CRITICAL_B11_THRESHOLD: i64 = 60;

const NAMES: &[&str] = &[
    "حسن العامري",
    "فاطمة الزهراء",
    "محمد عبدالله",
    "علي كريم",
    "زينب حسين",
    "عمر الشمري",
    "نور الهدى",
    "كريم الجبوري",
];
const BURIAL_TYPES: &[&str] = &["دفن عادي", "قبر مؤقت", "دفن جماعي"];
const DEATH_CAUSES: &[&str] = &["مرض طبيعي", "غير محدد", "حادث"];
// `record_id` MUST be the first column: `enkidw::kaki_generator::generate`
// derives every particle's uuid_hash from the FIRST data column's value
// (its own doc comment says so). civil_name was there originally -- with
// only 8 fixture names, that collapsed every generated row onto one of 8
// particles regardless of row count, caught directly by this crate's own
// round-trip test (`generated_rows_produce_real_distinct_particles_with_
// real_eav_attrs` failed 8 != 30 before this fix). record_id is the real
// row sequence number, unique by construction at any scale.
const HEADER: &str = "record_id,civil_name,date_of_birth,national_id,source_trust,validity_flag,\
completeness_score,arabic_density,geo_precision,address,grave_number,burial_type,\
death_cause,b11,epoch,state";

/// xorshift64star -- fast, deterministic, no crypto needed for synthetic
/// test data. Seeded per-row from a global seed XORed with the row index,
/// so any run with the same --seed reproduces byte-identical output.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn next_range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next_u64() % (hi - lo + 1) as u64) as i64
    }
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[(self.next_u64() as usize) % items.len()]
    }
}

struct Args {
    total_rows: u64,
    rows_per_file: u64,
    out_dir: PathBuf,
    zip: bool,
    seed: u64,
    progress_every: u64,
}

fn parse_args() -> Args {
    let mut total_rows: u64 = 1_000_000;
    let mut rows_per_file: u64 = 1_000_000;
    let mut out_dir = PathBuf::from("najaf_gen_out");
    let mut zip = false;
    let mut seed: u64 = 42;
    let mut progress_every: u64 = 1_000_000;

    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--total-rows" => total_rows = next_num(&mut it, "--total-rows"),
            "--rows-per-file" => rows_per_file = next_num(&mut it, "--rows-per-file"),
            "--out-dir" => {
                out_dir =
                    PathBuf::from(it.next().unwrap_or_else(|| fail("--out-dir needs a value")))
            }
            "--zip" => zip = true,
            "--seed" => seed = next_num(&mut it, "--seed"),
            "--progress-every" => progress_every = next_num(&mut it, "--progress-every"),
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => fail(&format!("unknown argument: {other}")),
        }
    }
    if rows_per_file == 0 {
        fail("--rows-per-file must be >= 1");
    }
    Args {
        total_rows,
        rows_per_file,
        out_dir,
        zip,
        seed,
        progress_every,
    }
}

fn next_num(it: &mut impl Iterator<Item = String>, flag: &str) -> u64 {
    it.next()
        .unwrap_or_else(|| fail(&format!("{flag} needs a value")))
        .parse()
        .unwrap_or_else(|_| fail(&format!("{flag} must be an integer")))
}

fn fail(msg: &str) -> ! {
    eprintln!("najaf-gen: {msg}");
    print_usage();
    std::process::exit(1);
}

fn print_usage() {
    eprintln!(
        "najaf-gen [--total-rows N] [--rows-per-file N] [--out-dir DIR] [--zip] \
         [--seed N] [--progress-every N]\n\
         Defaults: --total-rows 1000000 --rows-per-file 1000000 \
         --out-dir najaf_gen_out --seed 42 --progress-every 1000000\n\
         Example, 100M rows across 100 real STORE-zips of 1M rows each:\n  \
         najaf-gen --total-rows 100000000 --rows-per-file 1000000 --zip \
         --out-dir /data/najaf_100m"
    );
}

fn gen_row(rng: &mut Rng, seq: u64) -> String {
    let name = rng.pick(NAMES);
    let year = rng.next_range(1930, 2010);
    let month = rng.next_range(1, 12);
    let day = rng.next_range(1, 28);
    let national_id = format!("190{:06X}", rng.next_u64() % 0xFFFFFF);
    let source_trust = if rng.next_range(0, 9) > 7 {
        "Authoritative"
    } else {
        "Official"
    };
    let validity_flag = rng.next_range(0, 9) > 1;
    let completeness = 0.5 + (rng.next_range(0, 480) as f64) / 1000.0;
    let arabic_density = 0.6 + (rng.next_range(0, 300) as f64) / 1000.0;
    let geo_precision = format!("H3-r9-89c{:04X}", rng.next_u64() % 0xFFFF);
    let address = "النجف الأشرف، قسم الشهداء";
    let grave_number = format!("0941-{:03}", rng.next_range(0, 999));
    let burial_type = rng.pick(BURIAL_TYPES);
    let death_cause = rng.pick(DEATH_CAUSES);

    // b11: same shape distribution the HTML prototype and beemm_theater.gd
    // both already use (real thresholds, see this file's header).
    let roll = rng.next_range(0, 99);
    let b11: i64 = if roll > 80 {
        200 + rng.next_range(0, 39)
    } else if roll > 60 {
        140 + rng.next_range(0, 59)
    } else if roll > 15 {
        100 + rng.next_range(0, 39)
    } else {
        rng.next_range(0, 99)
    };
    let state = if b11 >= TRIBE_B11 {
        "golden"
    } else if b11 < CRITICAL_B11_THRESHOLD {
        "dead"
    } else {
        "fuzzy"
    };

    format!(
        "{seq},{name},{year:04}-{month:02}-{day:02},{national_id},{source_trust},{validity_flag},\
{completeness:.3},{arabic_density:.3},{geo_precision},{address},{grave_number},{burial_type},\
{death_cause},{b11},{seq},{state}\n"
    )
}

fn main() {
    let args = parse_args();
    fs::create_dir_all(&args.out_dir).unwrap_or_else(|e| {
        eprintln!("FATAL: create_dir_all {}: {e}", args.out_dir.display());
        std::process::exit(1);
    });

    // Honest, printed-up-front cost estimate -- average generated row is
    // ~140-160 bytes; this uses 150 as a round mid-estimate. STORE-only
    // zip (see module doc) means --zip does not shrink this.
    let est_bytes = args.total_rows * 150;
    println!(
        "𒁾 najaf-gen — {} rows, {} rows/file, seed={}, est. ~{:.1} GB on disk{}",
        args.total_rows,
        args.rows_per_file,
        args.seed,
        est_bytes as f64 / 1_073_741_824.0,
        if args.zip {
            " (--zip wraps in STORE zips -- same size, not smaller)"
        } else {
            ""
        }
    );

    let t0 = Instant::now();
    let mut written: u64 = 0;
    let mut file_idx = 0usize;

    while written < args.total_rows {
        let this_file_rows = args.rows_per_file.min(args.total_rows - written);
        let stem = format!("najaf_{file_idx:05}");
        let mut buf = Vec::with_capacity((this_file_rows as usize) * 150 + HEADER.len() + 1);
        buf.extend_from_slice(HEADER.as_bytes());
        buf.push(b'\n');

        for i in 0..this_file_rows {
            let seq = written + i + 1;
            let mut rng = Rng::new(args.seed ^ seq.wrapping_mul(0x9E3779B97F4A7C15));
            buf.extend_from_slice(gen_row(&mut rng, seq).as_bytes());
        }

        if args.zip {
            let zip_bytes = enkidw::build_store_zip(&format!("{stem}.csv"), &buf);
            let path = args.out_dir.join(format!("{stem}.zip"));
            let mut f = BufWriter::new(File::create(&path).unwrap_or_else(|e| {
                eprintln!("FATAL: create {}: {e}", path.display());
                std::process::exit(1);
            }));
            f.write_all(&zip_bytes).unwrap();
        } else {
            let path = args.out_dir.join(format!("{stem}.csv"));
            let mut f = BufWriter::new(File::create(&path).unwrap_or_else(|e| {
                eprintln!("FATAL: create {}: {e}", path.display());
                std::process::exit(1);
            }));
            f.write_all(&buf).unwrap();
        }

        written += this_file_rows;
        file_idx += 1;

        if written % args.progress_every < args.rows_per_file {
            let elapsed = t0.elapsed().as_secs_f64();
            let rate = written as f64 / elapsed.max(0.001);
            println!(
                "  {written} / {} rows written ({rate:.0}/s, {elapsed:.1}s elapsed, {file_idx} files so far)",
                args.total_rows
            );
        }
    }

    let elapsed = t0.elapsed().as_secs_f64();
    println!(
        "✓ done: {written} rows across {file_idx} files in {elapsed:.1}s ({:.0}/s) → {}",
        written as f64 / elapsed.max(0.001),
        args.out_dir.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;

    fn buffer(n: u64, seed: u64) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(HEADER.as_bytes());
        buf.push(b'\n');
        for seq in 1..=n {
            let mut rng = Rng::new(seed ^ seq.wrapping_mul(0x9E3779B97F4A7C15));
            buf.extend_from_slice(gen_row(&mut rng, seq).as_bytes());
        }
        buf
    }

    #[test]
    fn generated_csv_parses_into_the_expected_number_of_real_records() {
        let buf = buffer(50, 1);
        let records = enkidw::parse_csv(&buf);
        assert_eq!(records.len(), 50);
        assert_eq!(records[0].headers, HEADER.split(',').collect::<Vec<_>>());
    }

    #[test]
    fn generated_rows_produce_real_distinct_particles_with_real_eav_attrs() {
        let buf = buffer(30, 2);
        let records = enkidw::parse_csv(&buf);
        let minter = KakiMinter::new(TribeId::from_u16(0x0099));

        let mut seen_kakis = std::collections::HashSet::new();
        for record in &records {
            let entry = enkidw::kaki_generate(&minter, record);
            seen_kakis.insert(*entry.particle.bytes());
            assert!(
                entry
                    .eav
                    .iter()
                    .any(|t| t.attr_hash == enkidw::kaki_generator::fnv1a_32(b"civil_name")),
                "expected a civil_name EAV attribute on every generated row"
            );
        }
        assert_eq!(
            seen_kakis.len(),
            30,
            "each source row must mint a distinct particle"
        );
    }

    #[test]
    fn generated_zip_round_trips_through_the_real_zip_engine() {
        let buf = buffer(20, 3);
        let zipped = enkidw::build_store_zip("najaf_00000.csv", &buf);
        let entries = enkidw::zip_extract_ok(&zipped);
        assert_eq!(entries.len(), 1);
        let records = enkidw::parse_csv(&entries[0].data);
        assert_eq!(records.len(), 20);
    }

    #[test]
    fn b11_and_state_columns_stay_consistent_with_the_real_thresholds() {
        let buf = buffer(2000, 4);
        let records = enkidw::parse_csv(&buf);
        let b11_idx = records[0].headers.iter().position(|h| h == "b11").unwrap();
        let state_idx = records[0]
            .headers
            .iter()
            .position(|h| h == "state")
            .unwrap();
        for r in &records {
            let b11: i64 = std::str::from_utf8(&r.values[b11_idx])
                .unwrap()
                .parse()
                .unwrap();
            let state = std::str::from_utf8(&r.values[state_idx]).unwrap();
            if b11 >= TRIBE_B11 {
                assert_eq!(state, "golden");
            } else if b11 < CRITICAL_B11_THRESHOLD {
                assert_eq!(state, "dead");
            } else {
                assert_eq!(state, "fuzzy");
            }
        }
    }
}
