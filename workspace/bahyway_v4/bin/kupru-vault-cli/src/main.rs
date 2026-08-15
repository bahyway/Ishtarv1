//! kupru-vault-cli — Sargon vault identity check, callable from a plain
//! shell/Ansible context (Rust's crypto stack has no shell equivalent,
//! so infra playbooks that want to gate on a real passport, rather than
//! running unauthenticated, need this).
//!
//! Thin wrapper over `kupru-vault::open_vault_and_authenticate` — the
//! one real implementation this codebase now has of the Sargon-vault
//! open+verify logic (see that crate's own header for why it exists as
//! a separate crate: this CLI is its second caller, not the first
//! duplicate).
//!
//! HONEST SCOPE: a Sargon-format vault
//! (`sargon-passport-manager`'s `user://sargon_vault.dat` byte layout)
//! is a PASSPORT store, not a generic secrets vault -- it holds signed
//! `SargonPassport` identities, not arbitrary TLS/SSH/API-token
//! material. So this tool answers exactly one real question: "does the
//! caller hold a vault with a validly-sealed passport at or above a
//! given privilege_level?" -- an AUTHENTICATION gate, not a
//! secrets-retrieval mechanism.
//!
//! USAGE (passphrase via env var only -- never a CLI arg, which would
//! leak into `ps`/shell history):
//!   KUPRU_VAULT_PASSPHRASE=... kupru-vault-cli check \
//!     --vault /path/to/sargon_vault.dat --min-privilege 5
//!
//! On success: prints one line to stdout --
//!   OK subject_kaki=<hex> realm=<realm> privilege_level=<N> passport_id=<uuid> expires_at=<unix_secs>
//! and exits 0. On any failure (missing file, wrong passphrase, no
//! passport meets the threshold, malformed vault): prints a clear reason
//! to stderr and exits 1. No secret material -- not the passphrase, not
//! decrypted plaintext -- is ever written to a file or logged.

use std::env;
use std::fs;
use std::process::ExitCode;

fn parse_flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] != "check" {
        eprintln!("usage: KUPRU_VAULT_PASSPHRASE=... kupru-vault-cli check --vault <path> --min-privilege <1-7>");
        return ExitCode::FAILURE;
    }

    let Some(vault_path) = parse_flag(&args, "--vault") else {
        eprintln!("ERROR: --vault <path> is required");
        return ExitCode::FAILURE;
    };
    let min_privilege: u8 = match parse_flag(&args, "--min-privilege") {
        Some(s) => match s.parse() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("ERROR: --min-privilege must be an integer 1-7, got '{s}'");
                return ExitCode::FAILURE;
            }
        },
        None => {
            eprintln!("ERROR: --min-privilege <1-7> is required");
            return ExitCode::FAILURE;
        }
    };

    let Ok(passphrase) = env::var("KUPRU_VAULT_PASSPHRASE") else {
        eprintln!("ERROR: KUPRU_VAULT_PASSPHRASE env var not set (never pass the passphrase as a CLI arg -- it would leak into `ps`/shell history)");
        return ExitCode::FAILURE;
    };

    let vault_bytes = match fs::read(&vault_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("ERROR: cannot read vault file {vault_path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    match kupru_vault::open_vault_and_authenticate(&vault_bytes, passphrase.as_bytes()) {
        Ok(id) if id.privilege_level >= min_privilege => {
            println!(
                "OK subject_kaki={} realm={} privilege_level={} passport_id={} expires_at={}",
                id.subject_kaki_hex, id.realm, id.privilege_level, id.passport_id, id.expires_at
            );
            ExitCode::SUCCESS
        }
        Ok(id) => {
            eprintln!(
                "DENIED: strongest passport in this vault has privilege_level={} (realm={}), which is below the required minimum {min_privilege}",
                id.privilege_level, id.realm
            );
            ExitCode::FAILURE
        }
        Err(reason) => {
            eprintln!("DENIED: {reason}");
            ExitCode::FAILURE
        }
    }
}
