//! The six verdicts (DRAFT stub -- signatures only, no cleansing logic yet)
pub enum Verdict {
    Match,
    Merge,
    Split,
    Alias,
    Conflict,
    Unresolved,
}
