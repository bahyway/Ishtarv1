//! template-library — built-in Default template catalog (§6.3).

pub mod defaults;

pub use defaults::{
    load_defaults, civil_registry_template, operational_template, sensor_stream_template,
    ATTR_STATE, ATTR_QUALITY, ATTR_COLOR_RGB, ATTR_FRESHNESS,
    ATTR_SNAPSHOT_DATE, ATTR_SNAPSHOT_STATE, ATTR_SNAPSHOT_FREQUENCY,
};
