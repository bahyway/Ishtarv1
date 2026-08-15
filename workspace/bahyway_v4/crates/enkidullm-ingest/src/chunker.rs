//! Semantic text chunker — splits extracted book text into context-preserving chunks.
//!
//! Strategy: sentence-boundary detection with configurable max tokens per chunk.
//! Sentence boundaries: `.` `!` `?` followed by whitespace + capital, or `\n\n`.
//! No external tokenizer — pure character-level heuristics.

/// A single text chunk with positional metadata.
#[derive(Debug, Clone)]
pub struct TextChunk {
    pub text:        String,
    pub char_offset: usize,   // Byte offset into original document
    pub word_count:  u32,
    pub chunk_index: u32,
}

/// Chunking configuration.
#[derive(Debug, Clone, Copy)]
pub struct ChunkConfig {
    /// Target words per chunk (will not split mid-sentence unless forced).
    pub target_words: u32,
    /// Hard limit — split here even if mid-sentence.
    pub max_words:    u32,
    /// Overlap: re-include this many words from end of previous chunk.
    pub overlap_words: u32,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self { target_words: 200, max_words: 400, overlap_words: 20 }
    }
}

/// Split text into semantic chunks.
#[allow(unused_assignments)]
pub fn chunk_text(text: &str, config: ChunkConfig) -> Vec<TextChunk> {
    if text.trim().is_empty() { return Vec::new(); }

    let sentences = split_sentences(text);
    let mut chunks = Vec::new();
    let mut buf = String::new();
    let mut buf_words = 0u32;
    let mut buf_offset = 0usize;
    let mut chunk_idx = 0u32;
    let mut overlap_tail: Vec<&str> = Vec::new();

    for (sent, offset) in &sentences {
        let sent_words = count_words(sent);

        // If adding this sentence would exceed target, flush current buffer
        if buf_words + sent_words > config.target_words && !buf.is_empty() {
            let words = count_words(&buf);
            chunks.push(TextChunk {
                text:        buf.trim().to_string(),
                char_offset: buf_offset,
                word_count:  words,
                chunk_index: chunk_idx,
            });
            chunk_idx += 1;

            // Compute overlap tail from this sentence
            overlap_tail = take_tail_words(sent, config.overlap_words);
            buf = overlap_tail.join(" ");
            buf.push(' ');
            buf_words = count_words(&buf);
            buf_offset = *offset;
        }

        // Hard limit: if single sentence exceeds max, split by words
        if sent_words > config.max_words {
            let parts = split_by_words(sent, config.max_words);
            for part in parts {
                let words = count_words(part);
                chunks.push(TextChunk {
                    text:        part.trim().to_string(),
                    char_offset: *offset,
                    word_count:  words,
                    chunk_index: chunk_idx,
                });
                chunk_idx += 1;
            }
            buf.clear();
            buf_words = 0;
        } else {
            if buf.is_empty() { buf_offset = *offset; }
            buf.push_str(sent);
            buf.push(' ');
            buf_words += sent_words;
        }
    }

    // Flush remaining
    if !buf.trim().is_empty() {
        let words = count_words(&buf);
        chunks.push(TextChunk {
            text:        buf.trim().to_string(),
            char_offset: buf_offset,
            word_count:  words,
            chunk_index: chunk_idx,
        });
    }

    chunks
}

/// Split text into (sentence, byte_offset) pairs.
fn split_sentences(text: &str) -> Vec<(String, usize)> {
    let mut sentences = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    while i < len {
        let b = bytes[i];

        // Double newline = paragraph boundary → always split
        if b == b'\n' && bytes.get(i + 1).copied() == Some(b'\n') {
            let s = text[start..i].trim().to_string();
            if !s.is_empty() { sentences.push((s, start)); }
            // Skip all following newlines
            while i < len && bytes[i] == b'\n' { i += 1; }
            start = i;
            continue;
        }

        // Sentence terminators: . ! ? followed by space/newline + uppercase or end
        if matches!(b, b'.' | b'!' | b'?') {
            let next = i + 1;
            if next >= len {
                let s = text[start..=i].trim().to_string();
                if !s.is_empty() { sentences.push((s, start)); }
                start = next;
            } else if bytes[next].is_ascii_whitespace() {
                // Look ahead for uppercase or numeric start of next sentence
                let mut j = next;
                while j < len && bytes[j].is_ascii_whitespace() { j += 1; }
                if j >= len || bytes[j].is_ascii_uppercase() || bytes[j].is_ascii_digit() {
                    let s = text[start..=i].trim().to_string();
                    if !s.is_empty() { sentences.push((s, start)); }
                    start = j;
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }

    // Remaining text
    if start < len {
        let s = text[start..].trim().to_string();
        if !s.is_empty() { sentences.push((s, start)); }
    }

    sentences
}

fn count_words(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

fn take_tail_words<'a>(text: &'a str, n: u32) -> Vec<&'a str> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let start = words.len().saturating_sub(n as usize);
    words[start..].to_vec()
}

#[allow(unused_assignments)]
fn split_by_words<'a>(text: &'a str, max: u32) -> Vec<&'a str> {
    let bytes = text.as_bytes();
    let mut parts = Vec::new();
    let mut word_count = 0u32;
    let mut start = 0;
    let mut in_word = false;
    let mut last_word_end = 0;

    for (i, &b) in bytes.iter().enumerate() {
        if b.is_ascii_whitespace() {
            if in_word {
                word_count += 1;
                last_word_end = i;
                if word_count >= max {
                    parts.push(&text[start..last_word_end]);
                    start = i;
                    word_count = 0;
                }
            }
            in_word = false;
        } else {
            in_word = true;
        }
    }
    if start < text.len() { parts.push(&text[start..]); }
    if parts.is_empty() { parts.push(text); }
    parts
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_short_text_single_chunk() {
        let text = "Hello world. This is a test. Short text.";
        let chunks = chunk_text(text, ChunkConfig { target_words: 100, max_words: 200, overlap_words: 5 });
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].text.contains("Hello"));
    }

    #[test]
    fn chunk_splits_on_paragraph_boundary() {
        let text = "First paragraph sentence one. Second sentence.\n\nSecond paragraph begins here.";
        let chunks = chunk_text(text, ChunkConfig { target_words: 3, max_words: 20, overlap_words: 0 });
        assert!(chunks.len() >= 2, "expected multiple chunks, got {}", chunks.len());
    }

    #[test]
    fn chunk_indices_sequential() {
        let text = "One. Two. Three. Four. Five. Six. Seven. Eight.";
        let chunks = chunk_text(text, ChunkConfig { target_words: 2, max_words: 4, overlap_words: 0 });
        for (i, c) in chunks.iter().enumerate() {
            assert_eq!(c.chunk_index, i as u32);
        }
    }

    #[test]
    fn chunk_word_count_accurate() {
        let text = "The quick brown fox jumps over the lazy dog.";
        let chunks = chunk_text(text, ChunkConfig { target_words: 100, max_words: 200, overlap_words: 0 });
        assert_eq!(chunks[0].word_count, 9);
    }

    #[test]
    fn empty_text_no_chunks() {
        let chunks = chunk_text("", ChunkConfig::default());
        assert!(chunks.is_empty());
    }

    #[test]
    fn whitespace_only_no_chunks() {
        let chunks = chunk_text("   \n\n   ", ChunkConfig::default());
        assert!(chunks.is_empty());
    }

    #[test]
    fn sentence_splitter_on_capitals() {
        let sents = split_sentences("Hello World. Another sentence. Final one.");
        assert_eq!(sents.len(), 3, "got {:?}", sents);
    }
}
