//! Real-document integration test for the EnkiduLLM ingest → embed chain.
//!
//! Existing unit tests in enkidullm-ingest and zikru-embed each cover their
//! own crate against synthetic byte literals / hand-built token hashes.
//! Nothing in the workspace previously chained a REAL file on disk through
//! ingest_file() -> tokenize() -> TrainingSample -> train_epoch() the way an
//! actual book-ingestion run would. This test does that using a real PDF
//! fixture (tests/fixtures/sample.pdf, a small Docker cheat-sheet PDF).
//!
//! Known limitation surfaced by this test, documented rather than hidden:
//! enkidullm-ingest's native PDF parser does not yet resolve font/glyph
//! encoding (no ToUnicode CMap support), so extracted text from a real PDF
//! is not always human-readable — this test asserts the pipeline produces
//! non-empty, tokenizable output and that training runs to completion, not
//! that the extracted text matches the source document's visible content.
//! Fixing PDF font-encoding resolution is a separate, much larger effort
//! (see playbook_250's header comment) and out of scope here.

use enkidullm_ingest::{ingest_file, tokenize, DocFormat};
use zikru_embed::{train_epoch, TrainingSample, ZikruEmbedModel};

const TRIBE_A: u16 = 0x1001;
const TRIBE_B: u16 = 0x1007;

#[test]
fn real_pdf_ingest_produces_nonempty_text() {
    let data = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sample.pdf"
    ))
    .expect("fixture PDF must be present");
    let result = ingest_file("sample.pdf", &data);

    assert_eq!(result.format, DocFormat::Pdf);
    assert!(
        !result.full_text.is_empty(),
        "native PDF parser extracted zero bytes of text"
    );
}

#[test]
fn real_pdf_to_trained_embedding_full_chain() {
    let data = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sample.pdf"
    ))
    .expect("fixture PDF must be present");
    let ingested = ingest_file("sample.pdf", &data);
    assert!(!ingested.full_text.is_empty());

    let tokens = tokenize(&ingested.full_text);
    assert!(
        tokens.len() >= 6,
        "expected at least 6 tokens from the real PDF to build anchor/positive/negative \
         samples, got {}",
        tokens.len()
    );

    let hashes: Vec<u32> = tokens.iter().map(|t| t.uuid_hash).collect();
    let third = hashes.len() / 3;

    let sample = TrainingSample {
        anchor_hashes: hashes[0..third.max(1)].to_vec(),
        positive_hashes: hashes[third.max(1)..(2 * third).max(2)].to_vec(),
        negative_hashes: hashes[(2 * third).max(2)..].to_vec(),
        anchor_tribe: TRIBE_A,
        negative_tribe: TRIBE_B,
    };
    assert!(!sample.anchor_hashes.is_empty());
    assert!(!sample.positive_hashes.is_empty());
    assert!(!sample.negative_hashes.is_empty());

    let mut model = ZikruEmbedModel::init(200, 8, TRIBE_A);
    assert_eq!(model.orbit.training_epochs, 0);

    let metrics = train_epoch(&mut model, &[sample], 1.0, 0.01, 0.9);

    assert_eq!(model.orbit.training_epochs, 1);
    assert_eq!(metrics.total_samples, 1);
    assert!(
        metrics.mean_loss.is_finite(),
        "loss went non-finite training on a real document"
    );
    assert!(metrics.mean_loss >= 0.0);
}
