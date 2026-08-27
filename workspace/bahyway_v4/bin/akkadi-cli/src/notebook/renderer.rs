use super::cell::{CellKind, CellState, NotebookCell};

pub fn render_cell(cell: &NotebookCell) {
    let kind_label = match &cell.kind {
        CellKind::AkkadiCommand => "[akkadi]",
        CellKind::AkkadianAol => "[.akk]",
        CellKind::HeptaMap => "[hepta]",
        CellKind::VgcaAnalysis => "[vgca]",
        CellKind::SqlQuery => "[sql]",
        CellKind::SovereignVsLegacy => "[compare]",
        CellKind::LiveStream => "[stream]",
        CellKind::Markdown => "[md]",
        CellKind::KakiQuery => "[kaki]",
        CellKind::PipelineStatus => "[pipeline]",
        CellKind::EnkiddbQuery => "[enkiddb]",
    };
    let state_sym = match cell.state {
        CellState::Idle => "○",
        CellState::Running => "◎",
        CellState::Done => "●",
        CellState::Error => "✗",
    };
    println!(
        "{state_sym} {kind_label} — {}",
        &cell.source[..cell.source.len().min(72)]
    );
    if let Some(out) = &cell.output {
        for line in out.lines() {
            println!("  │ {line}");
        }
    }
}

pub fn render_notebook_header(title: &str, author: &str, version: &str) {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  𒀭𒆳𒁺  {title:<42} ║");
    println!("║  Author: {author:<43} ║");
    println!("║  v{version:<50} ║");
    println!("╚══════════════════════════════════════════════════════╝");
}
