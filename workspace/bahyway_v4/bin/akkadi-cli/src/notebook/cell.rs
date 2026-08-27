use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellKind {
    AkkadiCommand,
    AkkadianAol,
    HeptaMap,
    VgcaAnalysis,
    SqlQuery,
    SovereignVsLegacy,
    LiveStream,
    Markdown,
    KakiQuery,
    PipelineStatus,
    /// A real HeptaScript `QUERY:`/`SEARCH:` request executed against a
    /// live EnkiDDB read node (see `kernel::AkkadiKernel::run_enkiddb_query`).
    /// Distinct from `SqlQuery` -- that variant's own name predates this
    /// one and its "EnkiDB" target was never wired to a real connection;
    /// this one always is.
    EnkiddbQuery,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CellState {
    #[default]
    Idle,
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CellMetadata {
    pub tags: Vec<String>,
    pub collapsed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCell {
    pub id: String,
    pub kind: CellKind,
    pub source: String,
    pub output: Option<String>,
    pub state: CellState,
    pub metadata: CellMetadata,
}

impl NotebookCell {
    pub fn new(kind: CellKind, source: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            kind,
            source: source.into(),
            output: None,
            state: CellState::default(),
            metadata: CellMetadata::default(),
        }
    }
}
