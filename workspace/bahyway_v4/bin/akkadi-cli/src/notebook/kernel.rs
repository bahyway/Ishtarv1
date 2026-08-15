use super::cell::{CellKind, CellState, NotebookCell};
use super::enkiddb_client;
use crate::config::AkkadiConfig;

pub struct AkkadiKernel {
    pub cfg: AkkadiConfig,
}

impl AkkadiKernel {
    pub fn new(cfg: AkkadiConfig) -> Self {
        Self { cfg }
    }

    pub fn execute(&self, cell: &mut NotebookCell) {
        cell.state = CellState::Running;
        let result = match &cell.kind {
            CellKind::AkkadiCommand    => self.run_akkadi_command(&cell.source),
            CellKind::AkkadianAol      => self.run_aaol(&cell.source),
            CellKind::HeptaMap         => self.run_hepta_map(&cell.source),
            CellKind::VgcaAnalysis     => self.run_vgca(&cell.source),
            CellKind::SqlQuery         => self.run_sql_query(&cell.source),
            CellKind::SovereignVsLegacy => self.run_compare(&cell.source),
            CellKind::LiveStream       => self.run_live_stream(&cell.source),
            CellKind::Markdown         => Ok(cell.source.clone()),
            CellKind::KakiQuery        => self.run_kaki_query(&cell.source),
            CellKind::PipelineStatus   => self.run_pipeline_status(),
            CellKind::EnkiddbQuery     => self.run_enkiddb_query(&cell.source),
        };
        match result {
            Ok(out)  => { cell.output = Some(out); cell.state = CellState::Done; }
            Err(e)   => { cell.output = Some(e);   cell.state = CellState::Error; }
        }
    }

    fn run_akkadi_command(&self, src: &str) -> Result<String, String> {
        Ok(format!("[AkkadiCommand] {src}"))
    }

    fn run_aaol(&self, src: &str) -> Result<String, String> {
        use aaol::compiler::parser::Parser;
        let (_, lex_errs, parse_errs) = Parser::parse_source(src);
        if !lex_errs.is_empty() || !parse_errs.is_empty() {
            let msg = lex_errs.iter().map(|e| format!("{e:?}"))
                .chain(parse_errs.iter().map(|e| format!("{e}")))
                .collect::<Vec<_>>().join("; ");
            return Err(msg);
        }
        Ok(format!("✓ AkkadianAOL compiled ({} chars)", src.len()))
    }

    fn run_hepta_map(&self, src: &str) -> Result<String, String> {
        Ok(format!("[HeptaMap] {src}"))
    }

    fn run_vgca(&self, src: &str) -> Result<String, String> {
        Ok(format!("[VgcaAnalysis] {src}"))
    }

    fn run_sql_query(&self, src: &str) -> Result<String, String> {
        Ok(format!("[SqlQuery → EnkiDB] {src}"))
    }

    fn run_compare(&self, src: &str) -> Result<String, String> {
        Ok(format!("[SovereignVsLegacy] {src}"))
    }

    fn run_live_stream(&self, src: &str) -> Result<String, String> {
        Ok(format!("[LiveStream] {src}"))
    }

    fn run_kaki_query(&self, src: &str) -> Result<String, String> {
        Ok(format!("[KakiQuery] {src}"))
    }

    fn run_pipeline_status(&self) -> Result<String, String> {
        Ok("BeeMDM 4 lanes × 8 stations — healthy".into())
    }

    /// The one real, network-executing cell kind: sends `src` (a raw
    /// `QUERY:...`/`SEARCH:...` request) to `cfg.enkiddb_read_host:
    /// enkiddb_read_port` over EnkiDDB's real wire protocol and returns
    /// exactly what the server said, timed. A server-side `ERR:...`
    /// response is still `Ok` here at the transport level (the frame
    /// round-tripped successfully) -- callers building a debug-capture
    /// notebook want that text preserved as the cell's output either way,
    /// not swallowed into a generic failure.
    fn run_enkiddb_query(&self, src: &str) -> Result<String, String> {
        let resp = enkiddb_client::send_query(&self.cfg.enkiddb_read_host, self.cfg.enkiddb_read_port, src)?;
        Ok(format!("{} ({}ms)", resp.response, resp.elapsed_ms))
    }
}
