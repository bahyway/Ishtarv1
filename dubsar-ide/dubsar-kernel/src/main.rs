mod kernel;
mod particle;
mod projection;
mod magics;
mod renderer;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("dubsar_kernel=info".parse()?))
        .init();

    let args: Vec<String> = std::env::args().collect();

    // Jupyter kernel protocol: called with connection file path
    // e.g. dubsar-kernel /tmp/kernel-abc123.json
    if args.len() < 2 {
        eprintln!("Usage: dubsar-kernel <connection-file>");
        eprintln!("       dubsar-kernel --install");
        eprintln!("       dubsar-kernel --demo");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "--install" => install_kernelspec().await,
        "--demo" => run_demo().await,
        connection_file => {
            tracing::info!("DubSar kernel starting with connection file: {}", connection_file);
            let config = kernel::ConnectionConfig::load(connection_file)?;
            let mut k = kernel::DubSarKernel::new(config).await?;
            k.run().await
        }
    }
}

async fn install_kernelspec() -> Result<()> {
    let kernel_dir = dirs_home()?.join(".local/share/jupyter/kernels/dubsar");
    std::fs::create_dir_all(&kernel_dir)?;

    let exe = std::env::current_exe()?;
    let spec = serde_json::json!({
        "argv": [exe.to_string_lossy(), "{connection_file}"],
        "display_name": "DubSar (Rust + Vulkan)",
        "language": "rust",
        "interrupt_mode": "signal",
        "metadata": {
            "debugger": false
        }
    });

    std::fs::write(
        kernel_dir.join("kernel.json"),
        serde_json::to_string_pretty(&spec)?,
    )?;

    println!("Installed DubSar kernel to {}", kernel_dir.display());
    println!("Restart VSCodium/Jupyter to pick it up.");
    Ok(())
}

async fn run_demo() -> Result<()> {
    use particle::ParticleCloud;
    use projection::Projection7D3D;
    use renderer::SoftwareRenderer;

    println!("=== DubSar IDE Demo: Storage Fragmentation Simulation ===");
    println!("Running with software renderer (Hyper-V compatible, no GPU passthrough needed)");
    println!();

    // Phase 1 demo: Storage Fragmentation with 1M particles
    let particle_count = 1_000_000;
    println!("Initialising {} particles in 7D space...", particle_count);
    let mut cloud = ParticleCloud::new_storage_fragmentation(particle_count);

    let projection = Projection7D3D::default();
    let renderer = SoftwareRenderer::new(800, 600)?;

    for step in 0..10 {
        cloud.step(0.016);
        let projected = cloud.project(&projection);
        let frame = renderer.render_frame(&projected, step)?;
        let path = format!("/tmp/dubsar_frame_{:04}.png", step);
        frame.save(&path)?;
        println!("  step {:02}: {} active particles → {}", step, cloud.active_count(), path);
    }

    println!();
    println!("Demo complete. Frames saved to /tmp/dubsar_frame_*.png");
    println!("Install kernel: dubsar-kernel --install");
    Ok(())
}

fn dirs_home() -> Result<std::path::PathBuf> {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .map_err(|_| anyhow::anyhow!("$HOME not set"))
}
