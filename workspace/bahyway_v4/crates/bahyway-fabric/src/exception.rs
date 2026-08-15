//! FabricException — structured error events, never silent failures.
//!
//! Every failure in the Fabric surfaces as a typed, auditable exception.
//! Exceptions carry enough context to route to the right handler:
//! dead-letter queue, alert, retry, or human review.

use crate::connector::SourceId;

/// Classification of the failure — determines the recovery strategy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExceptionKind {
    /// Source produced fields not declared in its SchemaContract.
    SchemaViolation,
    /// Required field missing from an incoming record.
    MissingRequiredField,
    /// A record failed H(P) quality threshold and cannot be promoted.
    QualityRejection,
    /// De-duplication detected a conflicting identity.
    DuplicateIdentity,
    /// Target refused delivery (e.g. capacity, auth, network).
    DeliveryFailure,
    /// Transformation stage returned an unexpected output shape.
    TransformError,
    /// Source extraction failed (connection lost, timeout, auth).
    ExtractionError,
    /// Internal Fabric logic error — should never happen in production.
    InternalFault,
}

/// A structured exception event emitted by the Fabric.
///
/// Exceptions are first-class citizens: logged to the Journal as EventKakis,
/// visible in Dubsar, and routable to any configured exception handler.
#[derive(Debug, Clone)]
pub struct FabricException {
    pub kind:       ExceptionKind,
    pub source_id:  Option<SourceId>,
    pub stage:      &'static str,
    pub message:    String,
    /// The raw record that triggered this exception (may be empty on extraction errors).
    pub payload:    Vec<(u32, Vec<u8>)>,
    /// Monotonic epoch at the time of the exception.
    pub epoch:      u32,
}

impl FabricException {
    pub fn schema_violation(
        source_id: SourceId,
        stage: &'static str,
        message: impl Into<String>,
        epoch: u32,
    ) -> Self {
        FabricException {
            kind: ExceptionKind::SchemaViolation,
            source_id: Some(source_id),
            stage,
            message: message.into(),
            payload: vec![],
            epoch,
        }
    }

    pub fn missing_field(
        source_id: SourceId,
        stage: &'static str,
        field_name: &str,
        record: Vec<(u32, Vec<u8>)>,
        epoch: u32,
    ) -> Self {
        FabricException {
            kind: ExceptionKind::MissingRequiredField,
            source_id: Some(source_id),
            stage,
            message: format!("Required field '{field_name}' absent"),
            payload: record,
            epoch,
        }
    }

    pub fn quality_rejection(
        source_id: SourceId,
        b11: u8,
        threshold: u8,
        record: Vec<(u32, Vec<u8>)>,
        epoch: u32,
    ) -> Self {
        FabricException {
            kind: ExceptionKind::QualityRejection,
            source_id: Some(source_id),
            stage: "hepta-score",
            message: format!("B11={b11} below quality threshold {threshold}"),
            payload: record,
            epoch,
        }
    }

    pub fn delivery_failure(
        stage: &'static str,
        message: impl Into<String>,
        epoch: u32,
    ) -> Self {
        FabricException {
            kind: ExceptionKind::DeliveryFailure,
            source_id: None,
            stage,
            message: message.into(),
            payload: vec![],
            epoch,
        }
    }

    pub fn extraction_error(
        source_id: SourceId,
        message: impl Into<String>,
        epoch: u32,
    ) -> Self {
        FabricException {
            kind: ExceptionKind::ExtractionError,
            source_id: Some(source_id),
            stage: "extraction",
            message: message.into(),
            payload: vec![],
            epoch,
        }
    }
}

impl std::fmt::Display for FabricException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] stage={} epoch={} — {}", self.kind, self.stage, self.epoch, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::SourceId;

    #[test]
    fn schema_violation_has_correct_kind() {
        let ex = FabricException::schema_violation(
            SourceId("erp.test"), "validation", "field X unknown", 1
        );
        assert_eq!(ex.kind, ExceptionKind::SchemaViolation);
        assert_eq!(ex.epoch, 1);
    }

    #[test]
    fn quality_rejection_encodes_b11() {
        let ex = FabricException::quality_rejection(
            SourceId("crm.test"), 55, 60, vec![], 7
        );
        assert!(ex.message.contains("B11=55"));
        assert!(ex.message.contains("60"));
    }

    #[test]
    fn display_includes_stage_and_epoch() {
        let ex = FabricException::delivery_failure("dw.deliver", "timeout", 99);
        let s = ex.to_string();
        assert!(s.contains("dw.deliver"));
        assert!(s.contains("99"));
    }
}
