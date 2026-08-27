//! Change context — who changed an .akk file, why, and what structurally changed.
//!
//! Every time Nabu regenerates documentation for an `.akk` source file, it
//! captures a `ChangeContext` describing the event.  This makes the codebase
//! itself queryable through Hepta:
//!   "Why did this pipeline change?  Who triggered it?  What declarations changed?"
//!
//! Stored as part of `DocumentationParticle` in `enki.code_documentation`.
//! Referenced by doc: Nabu_Markdown_Tool.md (Questions 03–04).

#![forbid(unsafe_code)]

/// What triggered the `.akk` file recompilation and documentation regeneration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriggerSource {
    /// Explicit `nabu compile` / `nabu doc` invocation by a human.
    Manual,
    /// Dependency-forced re-check — a referenced `.akk` source changed.
    CompilerCheck,
    /// A file-system watcher detected a change on disk.
    FileWatcher,
    /// Continuous integration build triggered the compile.
    Ci,
    /// A Data Steward forced regeneration via DubSar IDE override.
    StewardOverride,
    /// An explicit docs-generation request (e.g. `nabu doc --workspace`).
    DocsRequest,
    /// A referenced upstream particle's declaration changed (cascade).
    DependencyChange,
}

impl TriggerSource {
    /// Returns `true` if this trigger originated from automated infrastructure
    /// rather than a direct human action.
    pub fn is_automated(&self) -> bool {
        matches!(
            self,
            TriggerSource::CompilerCheck | TriggerSource::Ci | TriggerSource::DependencyChange
        )
    }

    /// Short label for Hepta query display.
    pub fn label(&self) -> &'static str {
        match self {
            TriggerSource::Manual => "Manual",
            TriggerSource::CompilerCheck => "CompilerCheck",
            TriggerSource::FileWatcher => "FileWatcher",
            TriggerSource::Ci => "CI",
            TriggerSource::StewardOverride => "StewardOverride",
            TriggerSource::DocsRequest => "DocsRequest",
            TriggerSource::DependencyChange => "DependencyChange",
        }
    }
}

/// Intent behind a change — recorded at compile time when known.
///
/// Defaults to `Unspecified` until the author annotates it.
/// MetaWay (v4.1) can suggest reasons from organizational policy documents.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChangeReason {
    /// A human developer edited the `.akk` source directly.
    UserEdit,
    /// A referenced upstream declaration changed and propagated here.
    DependencyPropagation,
    /// An organizational policy document required this change.
    PolicyUpdate,
    /// A defect correction (bug fix).
    CorrectionOfDefect,
    /// Structural improvement with no behavior change.
    Refactoring,
    /// A new capability was added.
    NewFeature,
    /// Security-driven change.
    SecurityPatch,
    /// Reason not recorded (default; can be refined later via DubSar).
    Unspecified,
}

impl ChangeReason {
    /// Returns `true` if this reason was explicitly provided by an author.
    pub fn is_documented(&self) -> bool {
        !matches!(self, ChangeReason::Unspecified)
    }

    /// Short label for Hepta query display and AlertWay documentation-gap alerts.
    pub fn label(&self) -> &'static str {
        match self {
            ChangeReason::UserEdit => "UserEdit",
            ChangeReason::DependencyPropagation => "DependencyPropagation",
            ChangeReason::PolicyUpdate => "PolicyUpdate",
            ChangeReason::CorrectionOfDefect => "CorrectionOfDefect",
            ChangeReason::Refactoring => "Refactoring",
            ChangeReason::NewFeature => "NewFeature",
            ChangeReason::SecurityPatch => "SecurityPatch",
            ChangeReason::Unspecified => "Unspecified",
        }
    }
}

/// Structural summary of what changed between two versions of an `.akk` file.
///
/// Populated by diffing the parsed AkkIR trees of successive versions.
/// All declaration names are static label slices — the actual KAKI references
/// are carried by the `DocumentationParticle.declares` / `.references` fields.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffSummary {
    /// Raw byte count added (including prose comments).
    pub bytes_added: u32,
    /// Raw byte count removed.
    pub bytes_removed: u32,
    /// Names of PARTICLE / TRIBE / RULE / FLOW / PIPELINE declarations that are new.
    pub declarations_added: Vec<String>,
    /// Names of declarations that were present in the prior version but removed.
    pub declarations_removed: Vec<String>,
    /// Names of declarations whose body changed (but whose name remained).
    pub declarations_modified: Vec<String>,
}

impl DiffSummary {
    /// Returns `true` if no structural declarations changed (prose-only diff).
    pub fn is_prose_only(&self) -> bool {
        self.declarations_added.is_empty()
            && self.declarations_removed.is_empty()
            && self.declarations_modified.is_empty()
    }

    /// Total declaration changes (add + remove + modify).
    pub fn total_declaration_changes(&self) -> usize {
        self.declarations_added.len()
            + self.declarations_removed.len()
            + self.declarations_modified.len()
    }
}

/// Full context of why and how an `.akk` documentation regeneration occurred.
///
/// Attached to every `DocumentationParticle` that differs from its predecessor.
/// `None` on the very first generation (no prior version to compare).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeContext {
    /// What automated or human event triggered the regeneration.
    pub triggered_by: TriggerSource,
    /// Intent behind the change (may be `Unspecified` if not annotated).
    pub reason: ChangeReason,
    /// Structural diff between this version and its predecessor.
    pub diff: DiffSummary,
}

impl ChangeContext {
    /// Create a minimal context when no richer information is available.
    pub fn unspecified(triggered_by: TriggerSource) -> Self {
        ChangeContext {
            triggered_by,
            reason: ChangeReason::Unspecified,
            diff: DiffSummary::default(),
        }
    }

    /// Returns `true` if the change is sufficiently documented for audit purposes.
    ///
    /// Audit-ready requires a non-`Unspecified` reason.
    pub fn is_audit_ready(&self) -> bool {
        self.reason.is_documented()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_source_labels_non_empty() {
        let sources = [
            TriggerSource::Manual,
            TriggerSource::CompilerCheck,
            TriggerSource::FileWatcher,
            TriggerSource::Ci,
            TriggerSource::StewardOverride,
            TriggerSource::DocsRequest,
            TriggerSource::DependencyChange,
        ];
        for s in &sources {
            assert!(!s.label().is_empty(), "label for {s:?} should not be empty");
        }
    }

    #[test]
    fn test_automated_triggers() {
        assert!(TriggerSource::CompilerCheck.is_automated());
        assert!(TriggerSource::Ci.is_automated());
        assert!(TriggerSource::DependencyChange.is_automated());
        assert!(!TriggerSource::Manual.is_automated());
        assert!(!TriggerSource::StewardOverride.is_automated());
    }

    #[test]
    fn test_change_reason_documented() {
        assert!(!ChangeReason::Unspecified.is_documented());
        assert!(ChangeReason::UserEdit.is_documented());
        assert!(ChangeReason::SecurityPatch.is_documented());
        assert!(ChangeReason::PolicyUpdate.is_documented());
    }

    #[test]
    fn test_change_reason_labels_non_empty() {
        let reasons = [
            ChangeReason::UserEdit,
            ChangeReason::DependencyPropagation,
            ChangeReason::PolicyUpdate,
            ChangeReason::CorrectionOfDefect,
            ChangeReason::Refactoring,
            ChangeReason::NewFeature,
            ChangeReason::SecurityPatch,
            ChangeReason::Unspecified,
        ];
        for r in &reasons {
            assert!(!r.label().is_empty(), "label for {r:?} should not be empty");
        }
    }

    #[test]
    fn test_diff_summary_prose_only() {
        let d = DiffSummary {
            bytes_added: 50,
            bytes_removed: 10,
            ..Default::default()
        };
        assert!(d.is_prose_only());
        assert_eq!(d.total_declaration_changes(), 0);
    }

    #[test]
    fn test_diff_summary_with_declarations() {
        let d = DiffSummary {
            bytes_added: 200,
            bytes_removed: 50,
            declarations_added: vec!["new_particle".to_string()],
            declarations_modified: vec!["existing_rule".to_string()],
            ..Default::default()
        };
        assert!(!d.is_prose_only());
        assert_eq!(d.total_declaration_changes(), 2);
    }

    #[test]
    fn test_change_context_unspecified_not_audit_ready() {
        let ctx = ChangeContext::unspecified(TriggerSource::Manual);
        assert!(!ctx.is_audit_ready());
    }

    #[test]
    fn test_change_context_documented_is_audit_ready() {
        let ctx = ChangeContext {
            triggered_by: TriggerSource::StewardOverride,
            reason: ChangeReason::PolicyUpdate,
            diff: DiffSummary::default(),
        };
        assert!(ctx.is_audit_ready());
    }

    #[test]
    fn test_change_context_dependency_propagation() {
        let ctx = ChangeContext {
            triggered_by: TriggerSource::DependencyChange,
            reason: ChangeReason::DependencyPropagation,
            diff: DiffSummary {
                declarations_modified: vec!["tribe_intake".to_string()],
                ..Default::default()
            },
        };
        assert!(ctx.is_audit_ready());
        assert!(ctx.triggered_by.is_automated());
        assert_eq!(ctx.diff.total_declaration_changes(), 1);
    }
}
