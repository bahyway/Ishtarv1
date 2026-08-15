use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Cuneiform,
}

impl OutputFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Table     => "table",
            Self::Json      => "json",
            Self::Cuneiform => "cuneiform",
        }
    }
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table"     => Ok(Self::Table),
            "json"      => Ok(Self::Json),
            "cuneiform" => Ok(Self::Cuneiform),
            other       => Err(format!("unknown format '{other}'; use table|json|cuneiform")),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
