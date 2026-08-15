// ============================================================================
// ga_orbit.comp.glsl — Geometric Algebra rotor orbit placement
// DubSar PDM prototype · compile: glslc -fshader-stage=comp -o ga_orbit.comp.spv
//
// Math: orbit plane bivector B = e_a ^ e_b in Cl(7), rotor R = exp(-B θ/2).
// The sandwich x' = R x R̃ rotates only the (a,b) components of x — exact GA,
// evaluated as a 2×2 rotation. The shape tablet carries (a,b); the GPU obeys.
// Same bivector encoding as the sealed Buzu format (GL-VIZ-001).
// ============================================================================
#version 450
layout(local_size_x = 256) in;

struct P {
    vec4 lo;      // hepta e0..e3
    vec4 hi;      // hepta e4..e6, w = phase offset
};

layout(std430, binding = 0) buffer Particles { P p[]; };

layout(push_constant) uniform PC {
    float theta;      // global orbital angle
    int   axis_a;     // bivector plane axis a (0..6)
    int   axis_b;     // bivector plane axis b (0..6)
    int   count;
} pc;

float get_axis(P q, int a) {
    return a < 4 ? q.lo[a] : q.hi[a - 4];
}
void set_axis(inout P q, int a, float v) {
    if (a < 4) q.lo[a] = v; else q.hi[a - 4] = v;
}

void main() {
    uint i = gl_GlobalInvocationID.x;
    if (i >= uint(pc.count)) return;

    P q = p[i];
    float ang = pc.theta + q.hi.w;          // rotor angle = global + phase

    // Rotor sandwich in plane e_a^e_b (B unit, B² = -1):
    //   x' = cos(θ)·x_ab + sin(θ)·(x rotated in plane), other axes fixed.
    float xa = get_axis(q, pc.axis_a);
    float xb = get_axis(q, pc.axis_b);
    float c = cos(ang), s = sin(ang);
    set_axis(q, pc.axis_a, c * xa - s * xb);
    set_axis(q, pc.axis_b, s * xa + c * xb);

    // NOTE: q.lo/q.hi now hold the particle's 7D Hepta position this tick.
    // The draw pass projects to screen (camera does 7D→3D→2D); the FROZEN
    // bench simply skips this dispatch for its scope (build guide Step 7).
    p[i] = q;
}
