//! template-library — built-in Default template catalog (§6.3).

pub mod defaults;

pub use defaults::{
    civil_registry_template, load_defaults, operational_template, sensor_stream_template,
    ATTR_COLOR_RGB, ATTR_FRESHNESS, ATTR_QUALITY, ATTR_SNAPSHOT_DATE, ATTR_SNAPSHOT_FREQUENCY,
    ATTR_SNAPSHOT_STATE, ATTR_STATE,
};
