// DubSar IDE — LOD downsampling compute shader (WGSL / WebGPU)
// Downsamples 1B particles → max_visible for the 3D viewport
// Runs in the VSCodium webview on bare-metal (WebGPU available)

struct Particle {
    position: vec3<f32>,
    scalar:   f32,
    momentum: vec3<f32>,
    tribe_id: u32,
}

struct VisiblePoint {
    pos:   vec3<f32>,  // projected 3D position
    color: vec4<f32>,  // rgba color by tribe + scalar
}

struct LODParams {
    total_count:  u32,
    max_visible:  u32,
    sample_seed:  u32,
    // 7D→3D projection (3 rows, padded to 4 cols for alignment)
    basis_x: vec4<f32>,
    basis_y: vec4<f32>,
    basis_z: vec4<f32>,
    scale:   f32,
    _pad:    vec3<f32>,
}

@group(0) @binding(0) var<storage, read>       particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> visible:   array<VisiblePoint>;
@group(0) @binding(2) var<uniform>             params:    LODParams;

fn lcg(seed: u32) -> u32 {
    return seed * 1664525u + 1013904223u;
}

fn tribe_color(tribe_id: u32, scalar: f32) -> vec4<f32> {
    switch tribe_id {
        case 0u: { return vec4(0.0, 0.8 + scalar * 0.2, 0.2, 0.8); }  // Tribe A — green
        case 1u: { return vec4(0.8 + scalar * 0.2, 0.1, 0.1, 0.8); }  // Tribe B — red
        default: { return vec4(1.0, 0.9 - scalar * 0.4, 0.0, 0.7); }  // Free — yellow
    }
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= params.total_count) { return; }

    // Reservoir-style probabilistic selection
    let sample_prob = f32(params.max_visible) / f32(params.total_count);
    let rng = lcg(idx ^ params.sample_seed);
    let threshold = f32(rng) / 4294967295.0;

    if (threshold > sample_prob) { return; }

    let out_idx = u32(threshold / sample_prob * f32(params.max_visible));
    if (out_idx >= params.max_visible) { return; }

    let p = particles[idx];

    // 7D state vector (extend pos to 7D with momentum + scalar)
    let v7 = array<f32, 7>(
        p.position.x, p.position.y, p.position.z,
        p.momentum.x, p.momentum.y, p.momentum.z,
        p.scalar
    );

    // Project 7D → 3D
    let px = params.basis_x.x * v7[0] + params.basis_x.y * v7[1] + params.basis_x.z * v7[2]
           + params.basis_x.w * v7[3]; // extended 4 dims only; rest is in basis_y/z
    let py = params.basis_y.x * v7[0] + params.basis_y.y * v7[1] + params.basis_y.z * v7[2]
           + params.basis_y.w * v7[3];
    let pz = params.basis_z.x * v7[0] + params.basis_z.y * v7[1] + params.basis_z.z * v7[2]
           + params.basis_z.w * v7[3];

    visible[out_idx] = VisiblePoint(
        vec3(px, py, pz) * params.scale,
        tribe_color(p.tribe_id, p.scalar)
    );
}
