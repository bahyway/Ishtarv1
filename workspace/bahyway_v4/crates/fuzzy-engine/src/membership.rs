// ============================================================================
// fuzzy-engine/src/membership.rs
// Trapezoidal and triangular membership functions
// ============================================================================
#![allow(missing_docs)]

/// Trapezoidal membership: flat top between b and c, slopes on a→b and c→d.
/// Returns 0.0–1.0.
pub fn f_trap(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    if x <= a || x >= d { return 0.0; }
    if x >= b && x <= c { return 1.0; }
    if x < b { return (x - a) / (b - a); }
    (d - x) / (d - c)
}

/// Triangular membership: peak at b, zero at a and c.
pub fn f_tri(x: f32, a: f32, b: f32, c: f32) -> f32 {
    f_trap(x, a, b, b, c)
}

/// Shoulder-left: full membership below b, slope b→c, zero above c.
pub fn f_left(x: f32, b: f32, c: f32) -> f32 {
    if x <= b { return 1.0; }
    if x >= c { return 0.0; }
    (c - x) / (c - b)
}

/// Shoulder-right: zero below a, slope a→b, full membership above b.
pub fn f_right(x: f32, a: f32, b: f32) -> f32 {
    if x >= b { return 1.0; }
    if x <= a { return 0.0; }
    (x - a) / (b - a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn trap_below_a_is_zero()      { assert_eq!(f_trap(0.0, 10.0, 20.0, 80.0, 90.0), 0.0); }
    #[test] fn trap_above_d_is_zero()      { assert_eq!(f_trap(100.0, 10.0, 20.0, 80.0, 90.0), 0.0); }
    #[test] fn trap_flat_top_is_one()      { assert_eq!(f_trap(50.0, 10.0, 20.0, 80.0, 90.0), 1.0); }
    #[test] fn trap_left_slope_midpoint()  { let v = f_trap(15.0, 10.0, 20.0, 80.0, 90.0); assert!((v - 0.5).abs() < 0.001); }
    #[test] fn trap_right_slope_midpoint() { let v = f_trap(85.0, 10.0, 20.0, 80.0, 90.0); assert!((v - 0.5).abs() < 0.001); }
    #[test] fn tri_peak_is_one()           { assert_eq!(f_tri(50.0, 0.0, 50.0, 100.0), 1.0); }
    #[test] fn tri_base_left_is_zero()     { assert_eq!(f_tri(0.0, 0.0, 50.0, 100.0), 0.0); }
    #[test] fn tri_base_right_is_zero()    { assert_eq!(f_tri(100.0, 0.0, 50.0, 100.0), 0.0); }
    #[test] fn left_shoulder_below_b()    { assert_eq!(f_left(20.0, 50.0, 80.0), 1.0); }
    #[test] fn left_shoulder_above_c()    { assert_eq!(f_left(90.0, 50.0, 80.0), 0.0); }
    #[test] fn right_shoulder_above_b()   { assert_eq!(f_right(90.0, 50.0, 80.0), 1.0); }
    #[test] fn right_shoulder_below_a()   { assert_eq!(f_right(20.0, 50.0, 80.0), 0.0); }
}
