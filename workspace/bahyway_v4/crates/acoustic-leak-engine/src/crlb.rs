//! The error budget. Every localization carries its own proven sigma_d:
//!
//! >   sigma_d^2 = (v/2)^2 * sigma_tau^2  +  dx^2 * (sigma_v/v)^2
//!
//! with the Cramer-Rao lower bound on delay estimation
//!
//! >   sigma_tau >= 1 / (2*pi*B*sqrt(B*T*SNR))
//!
//! (B usable bandwidth Hz, T observation seconds, SNR linear).
//! No algorithm may claim below this floor; the verdict layer enforces it.

#[derive(Debug, Clone, Copy)]
pub struct ErrorBudgetInput {
    pub v_mps: f64,
    pub sigma_v_rel: f64,
    pub bandwidth_hz: f64,
    pub observation_s: f64,
    pub snr_linear: f64,
    /// Leak distance from segment midpoint (m) -- scales the velocity term.
    pub dx_from_midpoint_m: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct ErrorBudget {
    pub sigma_tau_s: f64,
    pub timing_term_m: f64,
    pub velocity_term_m: f64,
    pub sigma_d_m: f64,
}

pub fn crlb_sigma_tau_s(bandwidth_hz: f64, observation_s: f64, snr_linear: f64) -> f64 {
    let b = bandwidth_hz.max(1e-9);
    let bt_snr = (b * observation_s.max(1e-9) * snr_linear.max(1e-9)).sqrt();
    1.0 / (2.0 * std::f64::consts::PI * b * bt_snr)
}

pub fn error_budget(inp: &ErrorBudgetInput) -> ErrorBudget {
    let sigma_tau = crlb_sigma_tau_s(inp.bandwidth_hz, inp.observation_s, inp.snr_linear);
    let timing = (inp.v_mps / 2.0) * sigma_tau;
    let velocity = inp.dx_from_midpoint_m.abs() * inp.sigma_v_rel;
    ErrorBudget {
        sigma_tau_s: sigma_tau,
        timing_term_m: timing,
        velocity_term_m: velocity,
        sigma_d_m: (timing * timing + velocity * velocity).sqrt(),
    }
}
