//! kidinnu-engine (name PROPOSED per NL-001) — GL-ALG-001-A2 DRAFT.
//! D*(z) = argmin_m max_Θ D_Θ(m), s.t. no full doors.
//! Lives are never averaged; ε > 0 always; the map proposes, command decrees.
pub mod types;
pub mod fadam;
pub mod minimax;
pub mod seal;
pub mod export;
