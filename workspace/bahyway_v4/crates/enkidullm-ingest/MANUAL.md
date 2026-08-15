# 𒂗𒆠𒁺 enkidullm-ingest — Manual
**Version:** 4.0.2 | **Layer:** 9.5 — EnkiduLLM Ingestion Pipeline

---

## What It Solves

Books arrive as binary files — PDF, Epub, or plain text. Before the system can reason about a book, it must extract plain text, identify structure, and break it into chunks small enough to embed.

This must happen **without any third-party parsing libraries**. No `pdf-extract`, no `zip`, no `flate2`. Epub files are ZIP archives containing DEFLATE-compressed XHTML. PDF files wrap text in content streams that may themselves be DEFLATE-compressed. Both formats require a native DEFLATE decompressor.

**`enkidullm-ingest` solves the full raw-bytes → tokenized-chunks pipeline in pure Rust.**

---

## How It Works (Mechanism)

The pipeline has five sequential stages:

```
[raw bytes]
    ↓  inflate.rs       — decompress DEFLATE blocks
    ↓  pdf.rs / epub.rs — extract plain text + metadata
    ↓  chunker.rs       — split text into semantic chunks
    ↓  tokenizer.rs     — tokenize chunks into TokenUnits
[TokenUnit stream ready for zikru-embed]
```

---

### Stage 1 — Native DEFLATE Decompressor (`inflate.rs`)

Implements RFC 1951 from scratch. Handles all three block types:

| BTYPE | Type | Usage |
|---|---|---|
| `00` | Stored (uncompressed) | Small Epub entries, test data |
| `01` | Fixed Huffman | Most real-world PDF/Epub data |
| `10` | Dynamic Huffman | High-compression Epub entries |

**Internal components:**

`BitReader` — 32-bit accumulator, LSB-first bit packing (RFC 1951 §3.1.1):
```
fill():      loads bytes into buf at current bit offset
read_bits(n): extracts n LSB bits, advances the accumulator
align_to_byte(): discards sub-byte padding; REWINDS pos by buffered bytes
                 so subsequent raw reads start at the correct stream position
```

`HuffDecoder` — canonical Huffman (RFC 1951 §3.2.2):
```
build(lengths[]):  computes first_code[] from symbol count histogram
decode(reader):    reads one bit at a time, accumulates MSB-first into code
                   matches against sorted (length, code, symbol) table
```

`inflate_zlib(data)` — strips 2-byte zlib header (CMF=0x78) and 4-byte Adler-32 trailer before inflating. Used by PDF FlateDecode streams.

**Critical invariant:** `align_to_byte()` must rewind `pos` by `bits/8` before clearing the buffer. The BitReader eagerly pre-loads up to 4 bytes; stored block LEN/NLEN must be read from the correct stream offset, not from where `fill()` left `pos`.

---

### Stage 2a — PDF Extractor (`pdf.rs`)

PDF text extraction without a full PDF parser:

1. Scan for `/Info` dictionary → extract `/Title`, `/Author`, `/Subject` metadata
2. Scan for all `stream` ... `endstream` pairs
3. For each stream: check preceding dictionary for `/FlateDecode` → decompress if needed
4. Call `extract_text_operators()` on the decoded content stream

**Text operator parser** handles the BT/ET block model:
```
BT          → begin text block (in_text = true)
ET          → end text block
(string) Tj → literal string output
[(s1)(s2)] TJ → array of strings with optional kerning integers
<hex> Tj    → hex-encoded string
T*          → newline
```

**Critical invariant:** `read_keyword()` returns `(empty, same_pos)` when the current character is a PDF delimiter (`/`, `<`, `(`, `[`). The main loop must advance `i` by at least 1 in this case or it loops forever. Fix: `i = if end > i { end } else { i + 1 }`.

---

### Stage 2b — Epub Extractor (`epub.rs`)

Epub is a ZIP archive. The reader scans for ZIP Local File Headers (signature `PK\x03\x04`):

```
Local File Header layout (30 bytes fixed):
  [4]  PK signature
  [2]  version needed
  [2]  flags
  [2]  compression method  (0=STORED, 8=DEFLATED)
  [2]  mod time / date
  [4]  CRC-32
  [4]  compressed size
  [4]  uncompressed size
  [2]  filename length (N)
  [2]  extra field length (M)
  [N]  filename
  [M]  extra field
  [K]  file data
```

For each entry: STORED → copy bytes directly; DEFLATED → call `inflate()`.

`.opf` entries are parsed by `parse_opf()` for Dublin Core metadata:
- `dc:title` → `EpubExtract.title`
- `dc:creator` → `EpubExtract.author`
- `dc:publisher` → `EpubExtract.publisher`
- `<dc:identifier opf:scheme="ISBN">` → `EpubExtract.isbn` (digits only, hyphens stripped)

`.xhtml` / `.html` entries are passed through `strip_html()` which:
- Removes all `<tag>` markup
- Decodes `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&nbsp;`, `&apos;`
- Collapses whitespace runs to single spaces

---

### Stage 3 — Semantic Chunker (`chunker.rs`)

Splits plain text into overlapping chunks suitable for embedding.

**Default `ChunkConfig`:**
```
target_words:  200   ← aim to fill chunks to this word count
max_words:     400   ← hard cap; oversized sentences are word-split
overlap_words:  20   ← words carried from previous chunk into next
```

**Sentence boundary detection** (`split_sentences`):
- Double newlines (`\n\n`) always split
- `.`, `!`, `?` followed by whitespace + uppercase letter splits
- Preserves character offset for provenance tracking

**Chunk output:**
```
TextChunk {
    text:        String    ← chunk plain text
    char_offset: usize     ← byte offset in original document
    word_count:  u32       ← word count of this chunk
    chunk_index: u32       ← sequential index 0, 1, 2, ...
}
```

---

### Stage 4 — TribalTokenizer (`tokenizer.rs`)

Byte-level tokenizer. No BPE. No vocabulary file. Tokens are classified into seven **TokenClass** values that map to the seven Hepta sectors (θ₀–θ₆):

| TokenClass | Value | Sector | Examples |
|---|---|---|---|
| Word | 0 | θ₀ | common words |
| ProperNoun | 1 | θ₁ | "CAP_Theorem", "ACID", "Kleppmann" |
| Operator | 2 | θ₂ | `+`, `-`, `=`, `→` |
| Number | 3 | θ₃ | `42`, `3.14`, `0xFF` |
| Terminal | 4 | θ₄ | `.`, `!`, `?` |
| Delimiter | 5 | θ₅ | `,`, `;`, `:`, `(`, `)` |
| Quotation | 6 | θ₆ | `"quoted text"` |

**ProperNoun detection:** ALLCAPS words or CamelCase words (e.g., `NetworkTopology`) are classified as ProperNoun.

**CamelCase splitting:** `NetworkTopology` → `["Network", "Topology"]` as separate tokens.

**Hash normalization:** token `uuid_hash` is computed on the lowercase, alphanumeric-only form — `"KAKI"` and `"kaki"` produce the same hash. This means embeddings are case-invariant at the token identity level.

---

### Top-Level API (`lib.rs`)

```rust
ingest_file(path: &str, data: &[u8]) -> IngestResult
```

Auto-detects format from file extension (`.pdf`, `.epub`, fallback → plain text). Returns:
```rust
IngestResult {
    title:      Option<String>,
    author:     Option<String>,
    isbn:       Option<String>,
    publisher:  Option<String>,
    full_text:  String,
    format:     DocFormat,   // Pdf | Epub | PlainText
}
```

---

## Dependency Map

```
enkidullm-ingest
    ├── enkidullm-core   ← KnowledgeTriple, IduState (future: BookKaki creation)
    └── bahyway-crc      ← CRC-16 (available for checksum validation of extracted data)
```

**Dependents:**
```
(application layer)  ← calls ingest_file(), then feeds chunks into zikru-embed
```

---

## Sovereign Constraints

| Rule | Location | Effect |
|---|---|---|
| **No third-party runtime deps** | `Cargo.toml` | No `flate2`, no `zip`, no `pdf-extract`, no `epub`. All decompression and parsing is native. |
| **No unsafe code** | `#![forbid(unsafe_code)]` | BitReader operates on safe slices only. |
| **RFC 1951 compliance** | `inflate.rs` | All three DEFLATE block types handled. zlib header (0x78) stripped. Adler-32 trailer stripped. |
| **Case-invariant token hashing** | `tokenizer.rs` | `normalize()` lowercases before FNV-1a hashing so embedding lookup is case-insensitive. |
| **Provenance preservation** | `chunker.rs` | Every `TextChunk` carries `char_offset` back to the original document so evidence can be traced to source page/position. |

---

---

## Test Coverage

### Depth Levels

| Level | Name | What It Proves |
|---|---|---|
| **L1** | Unit | Single function in isolation — one input, one expected output |
| **L2** | Component | Module behavior through sequential or stateful interaction |
| **L3** | Invariant | A sovereign rule that must never be violated |
| **L4** | End-to-End | Full pipeline across all modules in the crate |

### Test Cases — `inflate.rs` (5 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `empty_stored_block` | L1 | BTYPE=00 block with LEN=0 produces empty output without panic |
| `stored_block_roundtrip` | L2 | BTYPE=00 block with b"hello" decompresses exactly to b"hello" |
| `inflate_stored_large` | L2 | Two chained BTYPE=00 blocks (BFINAL=0 then BFINAL=1) are both decompressed and concatenated |
| `inflate_fixed_huffman` | L2 | BTYPE=01 (fixed Huffman) stream for b"aaaaaa" decodes to the correct 6-byte literal sequence |
| `inflate_zlib_wrapper` | **L3** | `inflate_zlib()` strips CMF header (0x78) and Adler-32 trailer, then inflates correctly — validates the full PDF FlateDecode path |

**Critical invariants tested:**
- `stored_block_roundtrip` and `inflate_stored_large` together prove that `align_to_byte()` correctly rewinds `pos` by buffered bytes before raw LE16 reads.
- `inflate_fixed_huffman` proves that the bit reader's LSB-first accumulation is compatible with Huffman codes transmitted MSB-first.

### Test Cases — `pdf.rs` (8 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `find_seq_basic` | L1 | `find_seq()` byte-search returns correct offset and None when absent |
| `parse_pdf_literal_string` | L1 | `(Designing Data-Intensive Applications)` → correct string, correct end offset |
| `parse_hex_string_basic` | L1 | `<4142>` → "AB" (hex decode of ASCII codes) |
| `parse_nested_parentheses` | L1 | `(foo (bar) baz)` → balanced-paren parser returns all text including inner content |
| `extract_simple_text_stream` | L2 | `BT (Hello World) Tj ET` → text operator parser extracts "Hello" and "World" |
| `extract_tj_array` | L2 | `BT [(Hello) 10 ( World)] TJ ET` → TJ array operator extracts both strings, ignores kerning integers |
| `empty_pdf_does_not_panic` | L2 | `extract_pdf(b"")` returns empty result without panic or infinite loop |
| `minimal_pdf_extraction` | **L4** | Full PDF byte sequence (obj, stream, endstream, endobj) → `full_text` contains "Hello PDF" — validates the complete stream scanner path |

**Critical invariant tested:**
- `minimal_pdf_extraction` is the key test for the infinite-loop fix: the PDF contains `/F1` (PDF name object starting with `/`). If `read_keyword()` returns empty and `i` is not advanced by 1, the test hangs forever. Passing proves the fix holds.

### Test Cases — `epub.rs` (6 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `strip_html_basic` | L1 | `<p>Hello <b>World</b></p>` → "Hello World" with no angle brackets remaining |
| `strip_html_entities` | L1 | `Tom &amp; Jerry` → "Tom & Jerry" (entity decoding) |
| `extract_xml_text_basic` | L1 | `<dc:title>Designing...</dc:title>` → correct title string |
| `extract_xml_text_with_attrs` | L1 | `<dc:creator opf:role="aut">Martin Kleppmann</dc:creator>` → author extracted despite attributes on the tag |
| `empty_epub_does_not_panic` | L2 | `extract_epub(b"")` returns empty result without panic |
| `zip_stored_entry_roundtrip` | L2 | Manually constructed ZIP (STORED method) → correct filename and content extracted |
| `isbn_extraction_from_opf` | **L3** | `<dc:identifier opf:scheme="ISBN">978-1491903698</dc:identifier>` → "9781491903698" (hyphens stripped, scheme matched) — validates the OPF ISBN detection path including the `<` prefix fix |

**Critical invariant tested:**
- `isbn_extraction_from_opf` verifies that `find_isbn_identifier()` searches for `<dc:identifier` (including `<`), so `extract_xml_text()` can find the opening tag in the chunk. Passing proves the off-by-one chunk-start bug is fixed.

### Test Cases — `chunker.rs` (7 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `empty_text_no_chunks` | L1 | Empty string → empty `Vec<TextChunk>` |
| `whitespace_only_no_chunks` | L1 | Whitespace-only string → empty Vec (treated as empty) |
| `chunk_short_text_single_chunk` | L1 | Text under `target_words` → exactly one chunk |
| `sentence_splitter_on_capitals` | L1 | `. T` pattern (period + space + uppercase) → sentence boundary detected |
| `chunk_word_count_accurate` | L2 | `word_count` field in each chunk matches actual word count of `text` field |
| `chunk_indices_sequential` | L2 | `chunk_index` values across all returned chunks are 0, 1, 2, ... with no gaps |
| `chunk_splits_on_paragraph_boundary` | L2 | Double-newline `\n\n` always produces a chunk boundary regardless of word count |

### Test Cases — `tokenizer.rs` (11 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `empty_text_no_tokens` | L1 | Empty string → empty `Vec<TokenUnit>` |
| `hash_stable_for_same_token` | L1 | Same word string called twice → same `uuid_hash` both times |
| `number_detection` | L1 | "42" → `TokenClass::Number` |
| `operator_detection` | L1 | "+" → `TokenClass::Operator` |
| `proper_noun_detection` | L1 | "CAP_Theorem" → `TokenClass::ProperNoun` |
| `camel_case_word` | L2 | "NetworkTopology" → two tokens: "Network" and "Topology" |
| `byte_offset_correct` | L2 | `byte_offset` on each token matches its position in the original string |
| `tokenize_simple_sentence` | L2 | Multi-word sentence produces correct token sequence with correct classes |
| `token_stats_complexity` | L2 | `lexical_complexity()` returns value in [0,1] range |
| `token_stats_high_complexity` | L2 | Text with many ProperNouns and Operators scores higher complexity than plain text |
| `hash_case_normalized` | **L3** | "KAKI" and "kaki" produce the **same** uuid_hash — proves case normalization before FNV-1a hashing |

### Test Cases — `lib.rs` (3 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `ingest_plain_text` | **L4** | `.txt` extension → `DocFormat::PlainText`, full_text contains the input content |
| `ingest_pdf_detects_format` | **L4** | `.pdf` extension → `DocFormat::Pdf`, `extract_pdf()` path exercised |
| `ingest_epub_detects_format` | **L4** | `.epub` extension → `DocFormat::Epub`, `extract_epub()` path exercised |

**Total: 41 tests** — L1: 16 · L2: 19 · L3: 3 · L4: 3

### Gaps & Future Test Targets

| Area | Missing Coverage | Suggested Test |
|---|---|---|
| `inflate.rs` | BTYPE=10 (dynamic Huffman) not directly tested — only covered indirectly via zlib_wrapper | Add test: construct minimal dynamic-Huffman DEFLATE block, verify decode |
| `pdf.rs` | FlateDecode path (`inflate_zlib` inside `extract_pdf`) not tested end-to-end | Add test: construct PDF with FlateDecode stream containing known text |
| `epub.rs` | DEFLATED ZIP entries (method=8) not tested — only STORED (method=0) | Add test: manually inflate-compress an XHTML entry, verify strip_html on decoded output |
| `chunker.rs` | Overlap words not verified across chunk boundaries | Add test: confirm that last N words of chunk K appear at start of chunk K+1 |

---

*"The ingestion gate is the first sovereign act. What enters the system must be sealed."*
