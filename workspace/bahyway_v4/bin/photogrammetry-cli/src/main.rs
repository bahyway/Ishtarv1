//! photogrammetry-cli — CLI front end for `photogrammetry-ingest`.
//!
//! Subcommands:
//!   parse <file>
//!       Parse a .ply/.xyz/.csv/.las point cloud and print its stats as JSON.
//!   detect-voids <file> <voxel_size> <max_epsilon>
//!       Parse, voxel-downsample, run GeoEngine's H0/H1/H2 persistence,
//!       and print the void report as JSON.
//!   run-sfm <photos_dir> <output_point_cloud_path> [<voxel_size> <max_epsilon>] -- <command> [args...]
//!       Run a caller-configured external SfM engine (no engine ships
//!       with this workspace -- see photogrammetry-ingest::sfm's own
//!       module doc), then report the resulting point cloud's stats
//!       (and void report, if voxel_size/max_epsilon were given).
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use photogrammetry_ingest::sfm::{run_sfm, SfmConfig};
use photogrammetry_ingest::voids::{detect_voids, VoidReport};
use photogrammetry_ingest::{parse_point_cloud_file, PointCloud};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some((cmd, rest)) = args.split_first() else {
        usage();
        return ExitCode::FAILURE;
    };
    let result = match cmd.as_str() {
        "parse" => cmd_parse(rest),
        "detect-voids" => cmd_detect_voids(rest),
        "run-sfm" => cmd_run_sfm(rest),
        _ => {
            usage();
            return ExitCode::FAILURE;
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() {
    eprintln!("usage:");
    eprintln!("  photogrammetry-cli parse <file>");
    eprintln!("  photogrammetry-cli detect-voids <file> <voxel_size> <max_epsilon>");
    eprintln!("  photogrammetry-cli run-sfm <photos_dir> <output_point_cloud_path> [<voxel_size> <max_epsilon>] -- <command> [args...]");
}

fn cmd_parse(args: &[String]) -> Result<(), String> {
    let path = args.first().ok_or("parse: missing <file>")?;
    let cloud = parse_point_cloud_file(&PathBuf::from(path)).map_err(|e| e.to_string())?;
    println!("{}", cloud_json(&cloud));
    Ok(())
}

fn cmd_detect_voids(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err("detect-voids: usage: detect-voids <file> <voxel_size> <max_epsilon>".to_string());
    }
    let cloud = parse_point_cloud_file(&PathBuf::from(&args[0])).map_err(|e| e.to_string())?;
    let voxel_size: f64 = args[1].parse().map_err(|_| "voxel_size must be a number".to_string())?;
    let max_epsilon: f64 = args[2].parse().map_err(|_| "max_epsilon must be a number".to_string())?;
    let report = detect_voids(&cloud, voxel_size, max_epsilon);
    println!("{}", void_report_json(&report));
    Ok(())
}

fn cmd_run_sfm(args: &[String]) -> Result<(), String> {
    let sep = args.iter().position(|a| a == "--").ok_or("run-sfm: missing '--' before the engine command")?;
    let (positional, engine) = args.split_at(sep);
    let engine = &engine[1..]; // drop the "--" itself
    let (engine_cmd, engine_args) = engine.split_first().ok_or("run-sfm: no engine command given after '--'")?;

    if positional.len() < 2 {
        return Err("run-sfm: usage: run-sfm <photos_dir> <output_point_cloud_path> [<voxel_size> <max_epsilon>] -- <command> [args...]".to_string());
    }
    let photos_dir = PathBuf::from(&positional[0]);
    let output_point_cloud_path = PathBuf::from(&positional[1]);
    let voxel_and_epsilon: Option<(f64, f64)> = if positional.len() >= 4 {
        Some((
            positional[2].parse().map_err(|_| "voxel_size must be a number".to_string())?,
            positional[3].parse().map_err(|_| "max_epsilon must be a number".to_string())?,
        ))
    } else {
        None
    };

    let cfg = SfmConfig {
        command: engine_cmd.clone(),
        args: engine_args.to_vec(),
        output_point_cloud_path,
    };
    let cloud = run_sfm(&photos_dir, &cfg).map_err(|e| e.to_string())?;

    if let Some((voxel_size, max_epsilon)) = voxel_and_epsilon {
        let report = detect_voids(&cloud, voxel_size, max_epsilon);
        println!("{}", void_report_json(&report));
    } else {
        println!("{}", cloud_json(&cloud));
    }
    Ok(())
}

fn cloud_json(cloud: &PointCloud) -> String {
    let bounds = cloud.bounds();
    let bounds_json = match bounds {
        Some((min, max)) => format!(
            "{{\"min\":[{},{},{}],\"max\":[{},{},{}]}}",
            min[0], min[1], min[2], max[0], max[1], max[2]
        ),
        None => "null".to_string(),
    };
    format!(
        "{{\"point_count\":{},\"has_colors\":{},\"bounds\":{}}}",
        cloud.len(),
        cloud.colors.is_some(),
        bounds_json
    )
}

fn void_report_json(report: &VoidReport) -> String {
    format!(
        "{{\"input_point_count\":{},\"downsampled_point_count\":{},\"component_count\":{},\"void_count\":{}}}",
        report.input_point_count,
        report.downsampled_point_count,
        report.component_count(),
        report.void_count()
    )
}
