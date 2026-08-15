//! Manual on-demand runner: `cargo run --release -p buzu-gpu -- 1000000000`
//! (particle count optional, defaults to 10,000,000). Prints the same
//! honestly-labeled result the test does, but lets the Architect ask
//! for the real 1B-particle sweep on real hardware directly.

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000_000);

    println!("BUZU GPU run — requesting {n} particles for the throughput measurement...");

    match buzu_gpu::run(n) {
        Err(e) => {
            eprintln!("wgpu error: {e}");
            std::process::exit(1);
        }
        Ok(None) => {
            println!("No wgpu adapter available at all (no GPU, no software Vulkan/Metal/DX/GL fallback).");
            println!("This machine cannot answer the >1B particles/<1s condition — run on eriduous-vdi or another machine with a real graphics driver.");
        }
        Ok(Some(result)) => {
            println!("{result:#?}");
            if !result.correctness_passed {
                eprintln!("CORRECTNESS FAILED — GPU output does not match buzu-core's sealed CPU law. Do not trust the throughput number below.");
                std::process::exit(1);
            }
            match &result.adapter {
                buzu_gpu::AdapterKind::Hardware { name, backend } => {
                    println!(
                        "\nREAL HARDWARE ({name}, {backend}): {:.2} Mparticles/s over {} particles in {:?}.",
                        result.particles_per_sec / 1e6,
                        result.particles_benchmarked,
                        result.gpu_elapsed
                    );
                    let secs_for_1b = 1e9 / result.particles_per_sec;
                    println!(
                        "Extrapolated to 1,000,000,000 particles at this rate: {secs_for_1b:.3}s ({})",
                        if secs_for_1b < 1.0 { "MEETS the <1s condition" } else { "does NOT meet the <1s condition yet" }
                    );
                }
                buzu_gpu::AdapterKind::Software { name, backend } => {
                    println!(
                        "\nSOFTWARE ADAPTER ({name}, {backend}): {:.2} Mparticles/s -- this is software emulation, NOT a real GPU. Correctness is proven; this throughput number does NOT answer the >1B/<1s condition. Run on eriduous-vdi for that.",
                        result.particles_per_sec / 1e6
                    );
                }
            }
        }
    }
}
