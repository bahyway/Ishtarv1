// ── Pipeline domain types ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PipelineType {
    Water = 0,
    Oil = 1,
    Sewage = 2,
}

impl PipelineType {
    pub fn name(self) -> &'static str {
        match self {
            PipelineType::Water => "Water",
            PipelineType::Oil => "Oil",
            PipelineType::Sewage => "Sewage",
        }
    }

    /// Normalized type index [0.0, 1.0] for KAKI encoding.
    pub fn norm(self) -> f32 {
        self as u8 as f32 / 2.0
    }
}

// ── Pipe material ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipeMaterial {
    AsbestosCement,
    CastIron,
    DuctileIron,
    Plastic,
    Unknown,
}

impl PipeMaterial {
    /// Susceptibility to defect formation [0.0, 1.0].
    pub fn defect_susceptibility(self) -> f32 {
        match self {
            PipeMaterial::AsbestosCement => 0.90,
            PipeMaterial::CastIron => 0.65,
            PipeMaterial::DuctileIron => 0.35,
            PipeMaterial::Plastic => 0.15,
            PipeMaterial::Unknown => 0.70,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            PipeMaterial::AsbestosCement => "Asbestos Cement",
            PipeMaterial::CastIron => "Cast Iron",
            PipeMaterial::DuctileIron => "Ductile Iron",
            PipeMaterial::Plastic => "Plastic",
            PipeMaterial::Unknown => "Unknown",
        }
    }
}

// ── Defect severity levels ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum DefectSeverity {
    Negligible = 0,
    Minor = 1,
    Moderate = 2,
    Severe = 3,
    Critical = 4,
}

impl DefectSeverity {
    pub fn from_score(score: f32) -> Self {
        let s = score.clamp(0.0, 1.0);
        if s >= 0.90 {
            DefectSeverity::Critical
        } else if s >= 0.70 {
            DefectSeverity::Severe
        } else if s >= 0.45 {
            DefectSeverity::Moderate
        } else if s >= 0.20 {
            DefectSeverity::Minor
        } else {
            DefectSeverity::Negligible
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            DefectSeverity::Negligible => "Negligible",
            DefectSeverity::Minor => "Minor",
            DefectSeverity::Moderate => "Moderate",
            DefectSeverity::Severe => "Severe",
            DefectSeverity::Critical => "Critical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_type_norm_in_range() {
        for t in [PipelineType::Water, PipelineType::Oil, PipelineType::Sewage] {
            let n = t.norm();
            assert!(n >= 0.0 && n <= 1.0);
        }
    }

    #[test]
    fn susceptibility_ordering() {
        assert!(
            PipeMaterial::AsbestosCement.defect_susceptibility()
                > PipeMaterial::DuctileIron.defect_susceptibility()
        );
        assert!(
            PipeMaterial::DuctileIron.defect_susceptibility()
                > PipeMaterial::Plastic.defect_susceptibility()
        );
    }

    #[test]
    fn severity_from_score() {
        assert_eq!(DefectSeverity::from_score(0.95), DefectSeverity::Critical);
        assert_eq!(DefectSeverity::from_score(0.75), DefectSeverity::Severe);
        assert_eq!(DefectSeverity::from_score(0.50), DefectSeverity::Moderate);
        assert_eq!(DefectSeverity::from_score(0.25), DefectSeverity::Minor);
        assert_eq!(DefectSeverity::from_score(0.05), DefectSeverity::Negligible);
    }
}
