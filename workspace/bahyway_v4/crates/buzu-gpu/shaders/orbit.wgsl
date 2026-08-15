// BUZU orbit evaluation compute shader (GL-VIZ-001 §1, GPU path).
//
// One invocation per particle. Reads the shared ChunkHeader (nucleus +
// bivector, D1: amortized across every particle in the chunk) and this
// particle's phase (theta) from the SoA theta buffer, evaluates the
// GOLDEN parametric position via bivector exponentiation + rotor
// sandwich product, and writes it out. This is the actual claim
// GL-VIZ-001 §1 makes: per-frame traffic is theta (already resident),
// not a stream of positions -- the position below is COMPUTED, not
// transferred.
//
// The rotation formula (P0..P3, then the final vx/vy/vz combination)
// is the closed-form Cl(3,0) sandwich product v' = R v R~ for
// R = s + a*e01 + b*e02 + c*e12. Derived by hand, then validated (not
// trusted on derivation alone) via a throwaway Rust property test that
// compared this exact closed form against `Rotor::apply` -- which
// itself goes through bahyway-algebra's tested `Multivector::geometric`
// product -- across 2000 random rotor/vector pairs before being ported
// here (max error ~4.4e-16, i.e. exact agreement to machine precision).
// This shader is not an independent re-derivation; it is that proven
// formula transcribed verbatim. GPU-vs-CPU end-to-end agreement is
// re-checked every run by buzu_gpu::run()'s own correctness pass.

struct ChunkHeader {
    nucleus_x: f32,
    nucleus_y: f32,
    nucleus_z: f32,
    bivector_b01: f32,
    bivector_b02: f32,
    bivector_b12: f32,
    count: u32,
    checksum: u32,
}

struct Params {
    radius: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var<uniform> header: ChunkHeader;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> theta_buf: array<f32>;
@group(0) @binding(3) var<storage, read_write> out_buf: array<vec4<f32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= header.count) {
        return;
    }

    let b01 = header.bivector_b01;
    let b02 = header.bivector_b02;
    let b12 = header.bivector_b12;
    let mag = sqrt(b01 * b01 + b02 * b02 + b12 * b12);
    let ub01 = b01 / mag;
    let ub02 = b02 / mag;
    let ub12 = b12 / mag;

    // Canonical reference direction: the Hodge-dual normal of the
    // unit bivector, then Gram-Schmidt against e0 or e1 (whichever is
    // less aligned) -- mirrors Bivector::canonical_reference exactly.
    let n = vec3<f32>(ub12, -ub02, ub01);
    var candidate = vec3<f32>(1.0, 0.0, 0.0);
    if (abs(dot(candidate, n)) >= 0.9) {
        candidate = vec3<f32>(0.0, 1.0, 0.0);
    }
    let d = dot(candidate, n);
    let proj = candidate - d * n;
    let refdir = normalize(proj);
    let reference = refdir * params.radius;

    let theta = theta_buf[i];
    let half = theta * 0.5;
    let s = cos(half);
    let sn = sin(half);
    // Rotor R = s - sn * B_hat (scalar + bivector: a, b, c below).
    let a = -sn * ub01;
    let b = -sn * ub02;
    let c = -sn * ub12;

    let x = reference.x;
    let y = reference.y;
    let z = reference.z;

    let p0 = s * x + a * y + b * z;
    let p1 = s * y - a * x + c * z;
    let p2 = s * z - b * x - c * y;
    let p3 = c * x - b * y + a * z;

    let vx = s * p0 + a * p1 + b * p2 + c * p3;
    let vy = s * p1 - a * p0 + c * p2 - b * p3;
    let vz = s * p2 - b * p0 - c * p1 + a * p3;

    let pos = vec3<f32>(
        header.nucleus_x + vx,
        header.nucleus_y + vy,
        header.nucleus_z + vz,
    );
    out_buf[i] = vec4<f32>(pos, 0.0);
}
