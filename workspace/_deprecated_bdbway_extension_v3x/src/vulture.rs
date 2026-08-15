use pgrx::bgworkers::*;
use pgrx::prelude::*;
use std::time::Duration;

#[pg_guard]
pub extern "C" fn vulture_worker_main() {
    BackgroundWorker::wait_for_signal_after_startup();

    // The Vulture connects to the database to clean up old DNA
    let mut client = BackgroundWorker::connect_to_database("bdbway_extension", "akkad");

    loop {
        client.transaction(|txn| {
            // Logic: Delete Nodes from transient partition if a Gem version exists
            txn.execute(
                "DELETE FROM bdb_fabric_transient t
                 WHERE EXISTS (
                    SELECT 1 FROM bdb_fabric_permanent p
                    WHERE p.stable_uuid = t.stable_uuid
                 )",
                None, None,
            );
        });

        // Wait 60 seconds (The Vulture sleeps while the Storm builds)
        BackgroundWorker::wait_latch(Some(Duration::from_secs(60)));
    }
}
