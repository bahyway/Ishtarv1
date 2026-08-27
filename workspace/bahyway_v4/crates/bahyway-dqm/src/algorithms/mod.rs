pub mod jaro_winkler;
pub mod levenshtein;
pub mod soundex;
pub mod zscore;

pub use jaro_winkler::{jaro, jaro_winkler, jaro_winkler_p};
pub use levenshtein::{levenshtein, levenshtein_similarity};
pub use soundex::{soundex, soundex_str, sounds_like};
pub use zscore::{batch_z_scores, slice_stats, RunningStats};
