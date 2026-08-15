//! template-engine — named EAV schema registry for BahyWay particles (§6).

pub mod attr_type;
pub mod template;
pub mod registry;
pub mod change_context;

pub use attr_type::{AttrType, AttrTypeRegistry, AttrTypeSpec, TypeViolation};
pub use template::{
    FieldSpec, Template, TemplateOrigin, validate_required,
    LawStatus, VerifiableLaw, ILKUM_DOC, SHU_GUR_DOC,
};
pub use registry::TemplateRegistry;
pub use change_context::{TriggerSource, ChangeReason, DiffSummary, ChangeContext};
