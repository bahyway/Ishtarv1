//! Kittu Engine v1 daemon entry point.
//! Usage: kittu-engine [enkidb_addr] [alert_email] [show_dir]
//! Defaults: 192.168.122.107:7001, security@bahyway.local, ./runtime/show_alerts
#![forbid(unsafe_code)]
use std::env;

fn main() {
    eprintln!("[kittu-engine v1] starting");
    let enkidb = env::args()
        .nth(1)
        .unwrap_or_else(|| "192.168.122.107:7001".to_string());
    let email = env::args()
        .nth(2)
        .unwrap_or_else(|| "security@bahyway.local".to_string());
    let show_dir = env::args()
        .nth(3)
        .unwrap_or_else(|| "./runtime/show_alerts".to_string());
    eprintln!("[kittu] EnkiSDB={enkidb} | email={email} | show_dir={show_dir}");
    kittu_engine::poll::poll_loop(&enkidb, &show_dir, &email);
}
