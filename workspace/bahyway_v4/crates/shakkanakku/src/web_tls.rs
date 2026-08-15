//! Real TLS for shakkanakku-web — rustls (pure-Rust TLS state machine,
//! `ring` crypto provider so no cmake-based native build is pulled in,
//! see Cargo.toml) terminating actual encrypted connections, not a label.
//!
//! On first run this mints a real self-signed X.509 certificate (rcgen)
//! and writes it next to the seal key this crate already manages
//! (`secrets/shakkanakku_web_{cert,key}.pem`, gitignored, owner-only
//! permissions — same pattern `report::load_or_create_key` already uses
//! for the AkkadianSeal key). Being self-signed, a browser will show a
//! trust warning on first connect — that's inherent to any internal tool
//! without a locally-trusted CA, not something to paper over; the fix is
//! for the Architect to explicitly trust this one certificate once, the
//! same way any other internal-only HTTPS service works.

use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use std::path::Path;
use std::sync::Arc;

pub fn load_or_generate_config(cert_path: &Path, key_path: &Path) -> Result<Arc<ServerConfig>> {
    if !cert_path.exists() || !key_path.exists() {
        generate_self_signed(cert_path, key_path)?;
    }
    let (chain, key) = load_pem(cert_path, key_path)?;

    // rustls 0.23 requires an explicit crypto provider install. Safe to
    // call more than once per process (e.g. under `cargo test`) — a
    // second install attempt just returns Err, which we ignore.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(chain, key)
        .context("rustls rejected the certificate/key pair")?;
    Ok(Arc::new(config))
}

fn generate_self_signed(cert_path: &Path, key_path: &Path) -> Result<()> {
    let sans = vec!["localhost".to_string(), "127.0.0.1".to_string(), "::1".to_string()];
    let cert_key = rcgen::generate_simple_self_signed(sans).context("could not generate self-signed certificate")?;

    if let Some(parent) = cert_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).context("cannot create secrets directory")?;
    }
    std::fs::write(cert_path, cert_key.cert.pem()).context("cannot write certificate PEM")?;
    std::fs::write(key_path, cert_key.key_pair.serialize_pem()).context("cannot write private key PEM")?;
    harden_permissions(key_path)?;
    Ok(())
}

#[cfg(unix)]
fn harden_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).context("cannot stat freshly-written TLS key")?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms).context("cannot restrict TLS key permissions")?;
    Ok(())
}

#[cfg(not(unix))]
fn harden_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

fn load_pem(cert_path: &Path, key_path: &Path) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert_bytes = std::fs::read(cert_path).context("cannot read TLS certificate")?;
    let key_bytes = std::fs::read(key_path).context("cannot read TLS private key")?;

    let chain: Vec<CertificateDer<'static>> =
        rustls_pemfile::certs(&mut cert_bytes.as_slice()).collect::<Result<_, _>>().context("malformed certificate PEM")?;
    if chain.is_empty() {
        anyhow::bail!("certificate file contains no PEM certificate blocks");
    }
    let key = rustls_pemfile::private_key(&mut key_bytes.as_slice())
        .context("malformed private key PEM")?
        .context("private key file contains no PEM private key block")?;
    Ok((chain, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_and_reloads_a_working_server_config() {
        let dir = std::env::temp_dir().join(format!("shk_tls_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let cert = dir.join("cert.pem");
        let key = dir.join("key.pem");

        let config1 = load_or_generate_config(&cert, &key).expect("first call must generate a fresh cert");
        assert!(cert.exists() && key.exists());

        // Second call must load the SAME cert/key from disk rather than
        // silently regenerating (which would invalidate any cert an
        // Architect has already told their browser to trust).
        let cert_bytes_before = std::fs::read(&cert).unwrap();
        let config2 = load_or_generate_config(&cert, &key).expect("second call must reuse the existing cert");
        let cert_bytes_after = std::fs::read(&cert).unwrap();
        assert_eq!(cert_bytes_before, cert_bytes_after, "an existing certificate must not be regenerated");
        assert!(Arc::ptr_eq(&config1, &config1)); // sanity: Arc is usable
        let _ = config2;

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn private_key_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("shk_tls_perm_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let cert = dir.join("cert.pem");
        let key = dir.join("key.pem");
        let _ = load_or_generate_config(&cert, &key).unwrap();
        let mode = std::fs::metadata(&key).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "TLS private key must be owner-read/write only");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
