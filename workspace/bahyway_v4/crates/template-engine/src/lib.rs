//! template-engine — named EAV schema registry for BahyWay particles (§6).

pub mod attr_type;
pub mod change_context;
pub mod registry;
pub mod template;

pub use attr_type::{AttrType, AttrTypeRegistry, AttrTypeSpec, TypeViolation};
pub use change_context::{ChangeContext, ChangeReason, DiffSummary, TriggerSource};
pub use registry::TemplateRegistry;
pub use template::{
    validate_required, FieldSpec, LawStatus, Template, TemplateOrigin, VerifiableLaw, ILKUM_DOC,
    SHU_GUR_DOC,
};
