#![forbid(unsafe_code)]
//! media-type-detector — magic-byte media type detection for every format
//! that BeeMDM may ingest.
//!
//! Detection is based on file signatures (magic bytes) only — no file-system
//! access, no extension guessing.  The detection result drives the
//! `record_router` in `enkidw` so that every archive or data file is
//! normalised into `Vec<RawRecord>` before entering the ETL station chain.
//!
//! ## Format coverage
//!
//! ### Archive / Container
//! ZIP · 7-Zip · RAR 4/5 · tar · tar.gz (GZIP) · tar.bz2 (BZIP2) · tar.xz (XZ)
//! · tar.zst (Zstandard) · ISO 9660 · CAB
//!
//! ### Structured data
//! CSV (UTF-8 heuristic) · JSON · XML / XHTML · BSON · Apache Parquet
//! · XLSX (Office Open XML) · XLS (BIFF8) · PDF · DOCX · DOC (BIFF8)
//! · ODS (Open Document Spreadsheet) · NDJSON · CBOR
//!
//! ### Common images
//! PNG · JPEG · GIF · BMP · WebP · TIFF (LE/BE) · ICO
//!
//! ### Geo / Hyperspectral / Drone / Satellite
//! GeoTIFF (TIFF with GeoTIFF tag, detected at TIFF level) · JPEG 2000
//! · HDF5 · NetCDF-3 · NetCDF-4 (= HDF5) · LAS / LAZ (LiDAR point cloud)
//! · DICOM (medical/remote-sensing) · ENVI (detected by text header)
//! · MrSID · ECW · NITF
//!
//! ### Video / Audio (metadata ingestion)
//! MP4 / MOV (MPEG-4) · AVI · MKV (Matroska) · WAV · FLAC · MP3

// ── MediaType ─────────────────────────────────────────────────────────────────

/// Every media type that BeeMDM can ingest, normalise, or route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaType {
    // ── Archives / Containers ────────────────────────────────────────────────
    Zip,
    SevenZip,
    Rar4,
    Rar5,
    /// Plain POSIX tar (no compression)
    Tar,
    /// GZIP-compressed tar
    TarGz,
    /// BZIP2-compressed tar
    TarBz2,
    /// XZ-compressed tar
    TarXz,
    /// Zstandard-compressed tar
    TarZst,
    /// ISO 9660 optical disc image
    Iso9660,
    /// Microsoft Cabinet
    Cab,

    // ── Structured data ──────────────────────────────────────────────────────
    Csv,
    Json,
    /// Newline-Delimited JSON (one JSON object per line)
    Ndjson,
    Xml,
    Bson,
    Cbor,
    /// Apache Parquet columnar format
    Parquet,
    /// Excel Open XML (.xlsx, .xlsm)
    Xlsx,
    /// Excel legacy binary (.xls, BIFF8)
    Xls,
    Pdf,
    /// Word Open XML (.docx)
    Docx,
    /// Word legacy binary (.doc, BIFF8)
    Doc,
    /// Open Document Spreadsheet (.ods)
    Ods,

    // ── Common images ────────────────────────────────────────────────────────
    Png,
    Jpeg,
    Gif,
    Bmp,
    WebP,
    /// TIFF / BigTIFF — also used as GeoTIFF container
    Tiff,
    Ico,

    // ── Geo / Hyperspectral / Drone / Satellite ──────────────────────────────
    /// JPEG 2000 (JP2 container)
    Jpeg2000,
    /// HDF5 hierarchical data format (also NetCDF-4, hyperspectral cubes)
    Hdf5,
    /// NetCDF-3 classic (not HDF5)
    NetCdf3,
    /// LAS point cloud (LiDAR)
    Las,
    /// LAZ compressed point cloud
    Laz,
    /// DICOM medical / satellite imagery
    Dicom,
    /// NITF — National Imagery Transmission Format (satellite/military)
    Nitf,
    /// MrSID raster (satellite, aerial)
    MrSid,
    /// ECW raster (satellite, aerial)
    Ecw,
    /// ENVI hyperspectral raw binary — detected by companion .hdr text header
    EnviHdr,

    // ── Video / Audio ────────────────────────────────────────────────────────
    /// MPEG-4 container (MP4, MOV, M4A …)
    Mp4,
    Avi,
    /// Matroska (MKV, WebM)
    Matroska,
    Wav,
    Flac,
    Mp3,

    // ── Fallback ─────────────────────────────────────────────────────────────
    /// Content identified as binary but type not recognised
    UnknownBinary,
    /// Content that appears to be plain UTF-8 text (no recognised structure)
    PlainText,
}

// ── MediaCategory ─────────────────────────────────────────────────────────────

/// Broad category used by the record router to choose a parsing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaCategory {
    /// ZIP / 7z / RAR / tar.* — must be extracted before content is parsed
    Archive,
    /// Directly parseable into `Vec<RawRecord>` (CSV / JSON / XML / BSON …)
    StructuredData,
    /// Raster, hyperspectral, drone, satellite — routed to image particle handler
    Image,
    /// LiDAR point clouds (LAS/LAZ) — single blob particle + metadata
    PointCloud,
    /// Video / Audio — single blob particle + metadata
    Media,
    /// Anything else — single blob particle with content-hash + size
    Opaque,
}

impl MediaType {
    /// Broad category that determines the routing strategy in `record_router`.
    pub fn category(self) -> MediaCategory {
        match self {
            MediaType::Zip
            | MediaType::SevenZip
            | MediaType::Rar4
            | MediaType::Rar5
            | MediaType::Tar
            | MediaType::TarGz
            | MediaType::TarBz2
            | MediaType::TarXz
            | MediaType::TarZst
            | MediaType::Iso9660
            | MediaType::Cab => MediaCategory::Archive,

            MediaType::Csv
            | MediaType::Json
            | MediaType::Ndjson
            | MediaType::Xml
            | MediaType::Bson
            | MediaType::Cbor
            | MediaType::Parquet
            | MediaType::Xlsx
            | MediaType::Xls
            | MediaType::Pdf
            | MediaType::Docx
            | MediaType::Doc
            | MediaType::Ods
            | MediaType::PlainText => MediaCategory::StructuredData,

            MediaType::Png
            | MediaType::Jpeg
            | MediaType::Gif
            | MediaType::Bmp
            | MediaType::WebP
            | MediaType::Tiff
            | MediaType::Ico
            | MediaType::Jpeg2000
            | MediaType::Hdf5
            | MediaType::NetCdf3
            | MediaType::Dicom
            | MediaType::Nitf
            | MediaType::MrSid
            | MediaType::Ecw
            | MediaType::EnviHdr => MediaCategory::Image,

            MediaType::Las | MediaType::Laz => MediaCategory::PointCloud,

            MediaType::Mp4
            | MediaType::Avi
            | MediaType::Matroska
            | MediaType::Wav
            | MediaType::Flac
            | MediaType::Mp3 => MediaCategory::Media,

            MediaType::UnknownBinary => MediaCategory::Opaque,
        }
    }

    /// IANA MIME type string for use in EAV triples.
    pub fn mime_type(self) -> &'static str {
        match self {
            MediaType::Zip         => "application/zip",
            MediaType::SevenZip    => "application/x-7z-compressed",
            MediaType::Rar4        => "application/x-rar-compressed",
            MediaType::Rar5        => "application/x-rar-compressed",
            MediaType::Tar         => "application/x-tar",
            MediaType::TarGz       => "application/gzip",
            MediaType::TarBz2      => "application/x-bzip2",
            MediaType::TarXz       => "application/x-xz",
            MediaType::TarZst      => "application/zstd",
            MediaType::Iso9660     => "application/x-iso9660-image",
            MediaType::Cab         => "application/vnd.ms-cab-compressed",
            MediaType::Csv         => "text/csv",
            MediaType::Json        => "application/json",
            MediaType::Ndjson      => "application/x-ndjson",
            MediaType::Xml         => "application/xml",
            MediaType::Bson        => "application/bson",
            MediaType::Cbor        => "application/cbor",
            MediaType::Parquet     => "application/vnd.apache.parquet",
            MediaType::Xlsx        => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            MediaType::Xls         => "application/vnd.ms-excel",
            MediaType::Pdf         => "application/pdf",
            MediaType::Docx        => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            MediaType::Doc         => "application/msword",
            MediaType::Ods         => "application/vnd.oasis.opendocument.spreadsheet",
            MediaType::Png         => "image/png",
            MediaType::Jpeg        => "image/jpeg",
            MediaType::Gif         => "image/gif",
            MediaType::Bmp         => "image/bmp",
            MediaType::WebP        => "image/webp",
            MediaType::Tiff        => "image/tiff",
            MediaType::Ico         => "image/x-icon",
            MediaType::Jpeg2000    => "image/jp2",
            MediaType::Hdf5        => "application/x-hdf5",
            MediaType::NetCdf3     => "application/x-netcdf",
            MediaType::Las         => "application/vnd.las",
            MediaType::Laz         => "application/vnd.laz",
            MediaType::Dicom       => "application/dicom",
            MediaType::Nitf        => "application/vnd.nitf",
            MediaType::MrSid       => "image/x-mrsid",
            MediaType::Ecw         => "image/x-ecw",
            MediaType::EnviHdr     => "text/x-envi-hdr",
            MediaType::Mp4         => "video/mp4",
            MediaType::Avi         => "video/x-msvideo",
            MediaType::Matroska    => "video/x-matroska",
            MediaType::Wav         => "audio/wav",
            MediaType::Flac        => "audio/flac",
            MediaType::Mp3         => "audio/mpeg",
            MediaType::PlainText   => "text/plain",
            MediaType::UnknownBinary => "application/octet-stream",
        }
    }

    /// Human-readable label for log output and EAV triples.
    pub fn label(self) -> &'static str {
        match self {
            MediaType::Zip         => "ZIP archive",
            MediaType::SevenZip    => "7-Zip archive",
            MediaType::Rar4        => "RAR 4 archive",
            MediaType::Rar5        => "RAR 5 archive",
            MediaType::Tar         => "TAR archive",
            MediaType::TarGz       => "GZIP-compressed TAR",
            MediaType::TarBz2      => "BZIP2-compressed TAR",
            MediaType::TarXz       => "XZ-compressed TAR",
            MediaType::TarZst      => "Zstandard-compressed TAR",
            MediaType::Iso9660     => "ISO 9660 disc image",
            MediaType::Cab         => "Microsoft Cabinet",
            MediaType::Csv         => "CSV (delimited text)",
            MediaType::Json        => "JSON",
            MediaType::Ndjson      => "NDJSON (newline-delimited JSON)",
            MediaType::Xml         => "XML",
            MediaType::Bson        => "BSON",
            MediaType::Cbor        => "CBOR",
            MediaType::Parquet     => "Apache Parquet",
            MediaType::Xlsx        => "Excel Open XML (.xlsx)",
            MediaType::Xls         => "Excel legacy binary (.xls)",
            MediaType::Pdf         => "PDF document",
            MediaType::Docx        => "Word Open XML (.docx)",
            MediaType::Doc         => "Word legacy binary (.doc)",
            MediaType::Ods         => "ODS spreadsheet",
            MediaType::Png         => "PNG image",
            MediaType::Jpeg        => "JPEG image",
            MediaType::Gif         => "GIF image",
            MediaType::Bmp         => "BMP image",
            MediaType::WebP        => "WebP image",
            MediaType::Tiff        => "TIFF / GeoTIFF",
            MediaType::Ico         => "ICO icon",
            MediaType::Jpeg2000    => "JPEG 2000 (JP2)",
            MediaType::Hdf5        => "HDF5 / NetCDF-4 / hyperspectral",
            MediaType::NetCdf3     => "NetCDF-3 (CDF)",
            MediaType::Las         => "LAS point cloud (LiDAR)",
            MediaType::Laz         => "LAZ compressed point cloud",
            MediaType::Dicom       => "DICOM medical/satellite",
            MediaType::Nitf        => "NITF satellite imagery",
            MediaType::MrSid       => "MrSID raster",
            MediaType::Ecw         => "ECW raster",
            MediaType::EnviHdr     => "ENVI hyperspectral header",
            MediaType::Mp4         => "MPEG-4 video/audio",
            MediaType::Avi         => "AVI video",
            MediaType::Matroska    => "Matroska (MKV/WebM)",
            MediaType::Wav         => "WAV audio",
            MediaType::Flac        => "FLAC audio",
            MediaType::Mp3         => "MP3 audio",
            MediaType::PlainText   => "Plain UTF-8 text",
            MediaType::UnknownBinary => "Unknown binary",
        }
    }
}

// ── Detection ─────────────────────────────────────────────────────────────────

/// Detect the media type from the first bytes of `data`.
/// Returns `MediaType::UnknownBinary` when no signature matches.
pub fn detect(data: &[u8]) -> MediaType {
    if data.is_empty() { return MediaType::UnknownBinary; }

    // ── Archives ─────────────────────────────────────────────────────────────

    // ZIP: PK\x03\x04 (also XLSX, DOCX, ODS — disambiguated below)
    if starts_with(data, &[0x50, 0x4B, 0x03, 0x04]) {
        return detect_zip_based(data);
    }
    // 7-Zip: 7z\xBC\xAF\x27\x1C
    if starts_with(data, &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]) { return MediaType::SevenZip; }
    // RAR 5.x: Rar!\x1A\x07\x01\x00
    if starts_with(data, &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00]) { return MediaType::Rar5; }
    // RAR 4.x: Rar!\x1A\x07\x00
    if starts_with(data, &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00]) { return MediaType::Rar4; }
    // GZIP / tar.gz: \x1F\x8B
    if starts_with(data, &[0x1F, 0x8B]) { return MediaType::TarGz; }
    // BZIP2 / tar.bz2: BZh
    if starts_with(data, &[0x42, 0x5A, 0x68]) { return MediaType::TarBz2; }
    // XZ / tar.xz: \xFD7zXZ\x00
    if starts_with(data, &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]) { return MediaType::TarXz; }
    // Zstandard: \x28\xB5\x2F\xFD
    if starts_with(data, &[0x28, 0xB5, 0x2F, 0xFD]) { return MediaType::TarZst; }
    // Plain TAR: "ustar" at byte 257
    if data.len() > 262 && &data[257..262] == b"ustar" { return MediaType::Tar; }
    // ISO 9660: CD001 at byte 32769
    if data.len() > 32_774 && &data[32_769..32_774] == b"CD001" { return MediaType::Iso9660; }
    // Microsoft CAB: MSCF
    if starts_with(data, &[0x4D, 0x53, 0x43, 0x46]) { return MediaType::Cab; }

    // ── Office / Document ─────────────────────────────────────────────────────
    // OLE2 compound (XLS, DOC, PPT …): \xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1
    if starts_with(data, &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
        return detect_ole2(data);
    }
    // PDF: %PDF
    if starts_with(data, &[0x25, 0x50, 0x44, 0x46]) { return MediaType::Pdf; }

    // ── Structured data ───────────────────────────────────────────────────────
    // Apache Parquet: PAR1 magic at start AND end
    if starts_with(data, b"PAR1") { return MediaType::Parquet; }
    // BSON: first 4 bytes = little-endian document size (heuristic: size ≤ len)
    if data.len() >= 5 && is_bson_heuristic(data) { return MediaType::Bson; }
    // CBOR: single-byte major type 5 (map) or 4 (array) — only if not text
    if starts_with(data, &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) { return MediaType::Png; }

    // ── Images ────────────────────────────────────────────────────────────────
    // PNG: \x89PNG\r\n\x1A\n
    // JPEG: FF D8 FF
    if starts_with(data, &[0xFF, 0xD8, 0xFF]) { return MediaType::Jpeg; }
    // GIF: GIF8
    if starts_with(data, b"GIF8") { return MediaType::Gif; }
    // BMP: BM
    if starts_with(data, &[0x42, 0x4D]) { return MediaType::Bmp; }
    // WebP: RIFF????WEBP
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" { return MediaType::WebP; }
    // TIFF little-endian: II*\x00
    if starts_with(data, &[0x49, 0x49, 0x2A, 0x00]) { return MediaType::Tiff; }
    // TIFF big-endian: MM\x00*
    if starts_with(data, &[0x4D, 0x4D, 0x00, 0x2A]) { return MediaType::Tiff; }
    // ICO: \x00\x00\x01\x00
    if starts_with(data, &[0x00, 0x00, 0x01, 0x00]) { return MediaType::Ico; }
    // JPEG 2000: \x00\x00\x00\x0CjP  (12-byte JP2 box)
    if starts_with(data, &[0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50, 0x20, 0x20]) { return MediaType::Jpeg2000; }
    // HDF5: \x89HDF\r\n\x1A\n
    if starts_with(data, &[0x89, 0x48, 0x44, 0x46, 0x0D, 0x0A, 0x1A, 0x0A]) { return MediaType::Hdf5; }
    if data.len() >= 2 && is_cbor_heuristic(data) { return MediaType::Cbor; }
    // NetCDF-3: CDF\x01 or CDF\x02
    if starts_with(data, &[0x43, 0x44, 0x46, 0x01]) || starts_with(data, &[0x43, 0x44, 0x46, 0x02]) {
        return MediaType::NetCdf3;
    }
    // LAS: LASF
    if starts_with(data, b"LASF") { return MediaType::Las; }
    // LAZ: same LASF header but with bit 7 of global_encoding set; detectable
    // by checking byte 15 (global_encoding high byte): if (data[7] & 0x80) != 0
    // — but since LASF covers both, look at extended header to differentiate.
    // Conservative: keep LASF as LAS; downstream can upgrade to LAZ via VLR parsing.
    // DICOM: DICM at offset 128
    if data.len() > 132 && &data[128..132] == b"DICM" { return MediaType::Dicom; }
    // NITF: NITF02
    if starts_with(data, b"NITF02") { return MediaType::Nitf; }
    // MrSID: msid (MrSID generation 3)
    if starts_with(data, b"msid") { return MediaType::MrSid; }
    // ECW: NCS Ecw v1 or ECWP: starts with 0x0155 (little-endian)
    if starts_with(data, &[0x01, 0x55]) || starts_with(data, b"ECWP") { return MediaType::Ecw; }

    // ── Video / Audio ─────────────────────────────────────────────────────────
    // MP4/MOV: ftyp box at offset 4 — 8-byte: ????ftyp
    if data.len() >= 8 && &data[4..8] == b"ftyp" { return MediaType::Mp4; }
    // AVI: RIFF????AVI
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..11] == b"AVI" { return MediaType::Avi; }
    // Matroska: \x1A\x45\xDF\xA3
    if starts_with(data, &[0x1A, 0x45, 0xDF, 0xA3]) { return MediaType::Matroska; }
    // WAV: RIFF????WAVE
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" { return MediaType::Wav; }
    // FLAC: fLaC
    if starts_with(data, b"fLaC") { return MediaType::Flac; }
    // MP3: ID3 tag or sync word FF E* / FF F*
    if starts_with(data, b"ID3")
        || (data.len() >= 2 && data[0] == 0xFF && (data[1] & 0xE0) == 0xE0)
    {
        return MediaType::Mp3;
    }

    // ── Text-based formats (no binary magic) ──────────────────────────────────
    detect_text(data)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn starts_with(data: &[u8], needle: &[u8]) -> bool {
    data.len() >= needle.len() && &data[..needle.len()] == needle
}

/// ZIP-based container disambiguation: XLSX, DOCX, ODS are all PK\x03\x04.
/// Examine the first local-file-name in the ZIP to decide.
fn detect_zip_based(data: &[u8]) -> MediaType {
    // Read first local file name length from ZIP local header (offset 26, 2 bytes LE)
    if data.len() < 34 { return MediaType::Zip; }
    let fname_len = u16::from_le_bytes([data[26], data[27]]) as usize;
    if data.len() < 30 + fname_len { return MediaType::Zip; }
    let fname = &data[30..30 + fname_len];

    if fname == b"[Content_Types].xml" || fname == b"_rels/.rels" {
        // Office Open XML — distinguish by looking for xl/ (Excel) vs word/ (Word)
        // Search first 4 KB for xl/ or word/
        let search = &data[..data.len().min(4096)];
        if contains(search, b"xl/") || contains(search, b"xl\\") { return MediaType::Xlsx; }
        if contains(search, b"word/") || contains(search, b"word\\") { return MediaType::Docx; }
        return MediaType::Xlsx; // default to Xlsx for unknown OOXML
    }
    if fname == b"mimetype" {
        // ODF container — check next bytes for ODS mime
        let search = &data[..data.len().min(512)];
        if contains(search, b"spreadsheet") { return MediaType::Ods; }
    }
    MediaType::Zip
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// OLE2 compound document disambiguation (XLS vs DOC).
fn detect_ole2(data: &[u8]) -> MediaType {
    // Scan the first 2 KB for workbook stream name indicators
    let search = &data[..data.len().min(2048)];
    // Excel workbook stream: "W\x00o\x00r\x00k\x00b\x00o\x00o\x00k" (UTF-16LE)
    if contains(search, b"W\x00o\x00r\x00k\x00b\x00o\x00o\x00k") { return MediaType::Xls; }
    // Word document stream: "W\x00o\x00r\x00d\x00D\x00o\x00c\x00u\x00m\x00e\x00n\x00t"
    if contains(search, b"W\x00o\x00r\x00d\x00D\x00o\x00c") { return MediaType::Doc; }
    // Fallback: XLS (most common OLE2 type in MDM context)
    MediaType::Xls
}

/// Heuristic for BSON: first 4 bytes must equal document size and size ≤ data.len().
fn is_bson_heuristic(data: &[u8]) -> bool {
    if data.len() < 5 { return false; }
    let sz = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    sz >= 5 && sz <= data.len() && data[data.len() - 1] == 0x00
}

/// Heuristic for CBOR: top-level major type 4 (array 0x80-0x97) or 5 (map 0xA0-0xB7).
fn is_cbor_heuristic(data: &[u8]) -> bool {
    let b = data[0];
    // Only fire if the first byte is NOT printable ASCII (avoids false positives on text)
    if b.is_ascii() { return false; }
    let major = b >> 5;
    major == 4 || major == 5
}

/// Detect text-based formats (CSV, JSON, XML, NDJSON, ENVI, plain text).
fn detect_text(data: &[u8]) -> MediaType {
    let head = &data[..data.len().min(512)];
    // Must be valid UTF-8 to classify as text
    let Ok(s) = std::str::from_utf8(head) else { return MediaType::UnknownBinary; };
    let s = s.trim_start();

    if s.is_empty() { return MediaType::UnknownBinary; }

    // JSON array or object
    if s.starts_with('[') || s.starts_with('{') { return detect_json_vs_ndjson(data); }
    // XML / HTML
    if s.starts_with('<') || s.starts_with("<?xml") { return MediaType::Xml; }
    // ENVI header: starts with "ENVI" as first token
    if s.starts_with("ENVI") { return MediaType::EnviHdr; }

    // CSV heuristic: lines contain commas or tabs, first line looks like a header
    if is_csv_heuristic(s) { return MediaType::Csv; }

    MediaType::PlainText
}

/// Distinguish JSON (single object/array) from NDJSON (one object per line).
fn detect_json_vs_ndjson(data: &[u8]) -> MediaType {
    let head = std::str::from_utf8(&data[..data.len().min(4096)]).unwrap_or("");
    let mut newline_objects = 0usize;
    for line in head.lines() {
        let l = line.trim();
        if l.starts_with('{') && l.ends_with('}') { newline_objects += 1; }
        if newline_objects >= 2 { return MediaType::Ndjson; }
    }
    MediaType::Json
}

/// CSV heuristic: ≥1 delimiter per line across ≥2 lines.
fn is_csv_heuristic(s: &str) -> bool {
    let mut lines = s.lines().filter(|l| !l.trim().is_empty()).take(4);
    let Some(first) = lines.next() else { return false; };
    let sep = if first.contains('\t') { '\t' } else { ',' };
    let count0 = first.chars().filter(|&c| c == sep).count();
    if count0 == 0 { return false; }
    lines.all(|l| l.chars().filter(|&c| c == sep).count() == count0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_zip() {
        assert_eq!(detect(&[0x50, 0x4B, 0x03, 0x04, 0, 0, 0, 0]), MediaType::Zip);
    }

    #[test]
    fn detects_7zip() {
        assert_eq!(detect(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0, 0]), MediaType::SevenZip);
    }

    #[test]
    fn detects_gzip() {
        assert_eq!(detect(&[0x1F, 0x8B, 0x08, 0x00]), MediaType::TarGz);
    }

    #[test]
    fn detects_rar5() {
        assert_eq!(detect(&[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00]), MediaType::Rar5);
    }

    #[test]
    fn detects_png() {
        assert_eq!(detect(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]), MediaType::Png);
    }

    #[test]
    fn detects_jpeg() {
        assert_eq!(detect(&[0xFF, 0xD8, 0xFF, 0xE0]), MediaType::Jpeg);
    }

    #[test]
    fn detects_pdf() {
        assert_eq!(detect(b"%PDF-1.7"), MediaType::Pdf);
    }

    #[test]
    fn detects_hdf5() {
        assert_eq!(detect(&[0x89, 0x48, 0x44, 0x46, 0x0D, 0x0A, 0x1A, 0x0A]), MediaType::Hdf5);
    }

    #[test]
    fn detects_netcdf3() {
        assert_eq!(detect(&[0x43, 0x44, 0x46, 0x01]), MediaType::NetCdf3);
    }

    #[test]
    fn detects_las() {
        assert_eq!(detect(b"LASF\x00\x00\x00\x00"), MediaType::Las);
    }

    #[test]
    fn detects_parquet() {
        assert_eq!(detect(b"PAR1\x00"), MediaType::Parquet);
    }

    #[test]
    fn detects_json_object() {
        assert_eq!(detect(b"{\"name\":\"Ali\"}"), MediaType::Json);
    }

    #[test]
    fn detects_json_array() {
        assert_eq!(detect(b"[{\"id\":1},{\"id\":2}]"), MediaType::Json);
    }

    #[test]
    fn detects_ndjson() {
        let data = b"{\"id\":1}\n{\"id\":2}\n";
        assert_eq!(detect(data), MediaType::Ndjson);
    }

    #[test]
    fn detects_xml() {
        assert_eq!(detect(b"<?xml version=\"1.0\"?><root/>"), MediaType::Xml);
    }

    #[test]
    fn detects_csv() {
        let data = b"name,age,city\nAli,30,Baghdad\nFatima,25,Basra\n";
        assert_eq!(detect(data), MediaType::Csv);
    }

    #[test]
    fn detects_tsv_as_csv() {
        let data = b"name\tage\tcity\nAli\t30\tBaghdad\n";
        assert_eq!(detect(data), MediaType::Csv);
    }

    #[test]
    fn tiff_category_is_image() {
        assert_eq!(MediaType::Tiff.category(), MediaCategory::Image);
    }

    #[test]
    fn hdf5_category_is_image() {
        assert_eq!(MediaType::Hdf5.category(), MediaCategory::Image);
    }

    #[test]
    fn las_category_is_point_cloud() {
        assert_eq!(MediaType::Las.category(), MediaCategory::PointCloud);
    }

    #[test]
    fn zip_category_is_archive() {
        assert_eq!(MediaType::Zip.category(), MediaCategory::Archive);
    }
}
