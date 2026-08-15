use rayon::prelude::*;
use std::fs::File;
use csv::WriterBuilder;
use uuid::Uuid;
use rand::Rng;

fn main() {
    let total_records = 1_000_000;
    let batch_size = 100_000;
    let num_batches = total_records / batch_size;

    println!("--- BDBWay 1.0: Firing 8-way Parallel Generation Storm ---");

    (0..num_batches).into_par_iter().for_each(|b_idx| {
        let filename = format!("najaf_batch_{:03}.csv", b_idx + 1);
        let file = File::create(&filename).unwrap();
        let mut wtr = WriterBuilder::new().from_writer(file);

        for i in 0..batch_size {
            let record_id = b_idx * batch_size + i;
            let stable_uuid = Uuid::new_v4();

            // Simulation of NajafWay Data
            wtr.write_record(&[
                record_id.to_string(),
                stable_uuid.to_string(),
                format!("Person_{}", record_id), // Simplified for Rust-base test
                "Male".to_string(),
                "1980".to_string(),
                "43".to_string(),
                "2023-12-25".to_string(),
                "Najaf_City".to_string(), // Tribe Root
                "31.9850".to_string(),    // Lat
                "44.3050".to_string(),    // Lon
            ]).unwrap();
        }
        println!("✓ Batch {} complete.", b_idx + 1);
    });
}
