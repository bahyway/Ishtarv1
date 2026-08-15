//! LAW-ALE-2: the medium selects the mathematics.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipeMaterial {
    DuctileIron,
    CastIron,
    Steel,
    Copper,
    Pe, // polyethylene
    Pvc,
    AsbestosCement,
    VitrifiedClay,
    Concrete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverClass {
    /// Metal: non-dispersive, high v -- plain GCC-PHAT is valid.
    PlainGccPhat,
    /// Plastic: dispersive v(omega) -- dispersion-compensated correlation required.
    DispersionCompensated,
    /// Jointed ceramic/concrete: heavy multipath -- joint-indexed search
    /// (matched-field class); the answer space is quantized to joints.
    JointIndexed,
}

impl PipeMaterial {
    /// Nominal wave speed in m/s -- SEED VALUE ONLY (LAW-ALE-3: must be
    /// replaced by in-situ calibration before any verdict).
    pub fn nominal_velocity_mps(self) -> f64 {
        match self {
            PipeMaterial::DuctileIron => 1200.0,
            PipeMaterial::CastIron => 1250.0,
            PipeMaterial::Steel => 1400.0,
            PipeMaterial::Copper => 1300.0,
            PipeMaterial::Pe => 350.0,
            PipeMaterial::Pvc => 480.0,
            PipeMaterial::AsbestosCement => 1000.0,
            PipeMaterial::VitrifiedClay => 950.0,
            PipeMaterial::Concrete => 1050.0,
        }
    }

    pub fn is_dispersive(self) -> bool {
        matches!(self, PipeMaterial::Pe | PipeMaterial::Pvc)
    }

    pub fn is_jointed_short_segment(self) -> bool {
        matches!(
            self,
            PipeMaterial::VitrifiedClay | PipeMaterial::Concrete | PipeMaterial::AsbestosCement
        )
    }

    /// LAW-ALE-2 enforcement point.
    pub fn solver_class(self) -> SolverClass {
        if self.is_dispersive() {
            SolverClass::DispersionCompensated
        } else if self.is_jointed_short_segment() {
            SolverClass::JointIndexed
        } else {
            SolverClass::PlainGccPhat
        }
    }

    /// Usable leak-noise bandwidth (Hz) -- drives the Cramer-Rao bound.
    pub fn usable_bandwidth_hz(self) -> f64 {
        match self.solver_class() {
            SolverClass::PlainGccPhat => 750.0,
            SolverClass::DispersionCompensated => 180.0,
            SolverClass::JointIndexed => 400.0,
        }
    }
}
