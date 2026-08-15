// ══════════════════════════════════════════════════════════════════════════════
// nisaba_hologram.gdshader
// NISABA 𒀭𒉀 — Sovereign Hologram Shader
// BahyWay.Ecosystem v4.0 — DUB.SAR Bahaa Fadam
//
// Creates the electric blue holographic robotic appearance:
//   - Animated scan lines moving upward
//   - Fresnel edge glow (bright at silhouette edges)
//   - Subtle flicker for hologram authenticity
//   - Mode-driven colour switching (set via GDScript)
//   - Transparency with depth sorting
// ══════════════════════════════════════════════════════════════════════════════

shader_type spatial;
render_mode unshaded, cull_disabled, depth_draw_never, blend_add;

// ── Colour control (changed per mode by nisaba_mode_controller.gd) ──────────
uniform vec4  emission_color   : source_color = vec4(0.0, 0.82, 1.0, 1.0);
uniform vec4  edge_color       : source_color = vec4(0.25, 0.95, 1.0, 1.0);
uniform vec4  scanline_color   : source_color = vec4(0.0, 0.65, 0.9, 1.0);

// ── Scan line parameters ────────────────────────────────────────────────────
uniform float scanline_speed   : hint_range(0.1, 8.0)   = 1.6;
uniform float scanline_density : hint_range(10.0, 300.0) = 90.0;
uniform float scanline_width   : hint_range(0.0, 1.0)   = 0.35;

// ── Fresnel / edge glow ─────────────────────────────────────────────────────
uniform float fresnel_power    : hint_range(0.5, 8.0)   = 2.8;
uniform float edge_intensity   : hint_range(0.0, 3.0)   = 1.4;

// ── Opacity and flicker ─────────────────────────────────────────────────────
uniform float base_opacity     : hint_range(0.0, 1.0)   = 0.72;
uniform float flicker_speed    : hint_range(0.0, 20.0)  = 6.5;
uniform float flicker_amount   : hint_range(0.0, 0.3)   = 0.06;

// ── Mode pulse (chest core / crown sync) ────────────────────────────────────
uniform float pulse_speed      : hint_range(0.0, 10.0)  = 1.8;
uniform float pulse_amount     : hint_range(0.0, 1.0)   = 0.18;

// ── Hologram interference bands (adds realism) ──────────────────────────────
uniform float band_density     : hint_range(0.0, 50.0)  = 8.0;
uniform float band_speed       : hint_range(0.0, 2.0)   = 0.3;
uniform float band_intensity   : hint_range(0.0, 0.5)   = 0.12;

void fragment() {
    // ── Scan lines ──────────────────────────────────────────────────────────
    float scan_phase = VERTEX.y * scanline_density + TIME * scanline_speed;
    float scan_raw   = sin(scan_phase);
    float scanline   = smoothstep(1.0 - scanline_width, 1.0, scan_raw);

    // ── Hologram interference bands (slower, wider) ─────────────────────────
    float band_phase = VERTEX.y * band_density - TIME * band_speed;
    float band       = sin(band_phase) * band_intensity;

    // ── Fresnel edge glow ───────────────────────────────────────────────────
    float ndotv    = dot(normalize(NORMAL), normalize(VIEW));
    float fresnel  = pow(1.0 - clamp(ndotv, 0.0, 1.0), fresnel_power);
    fresnel        = clamp(fresnel * edge_intensity, 0.0, 1.0);

    // ── Flicker ─────────────────────────────────────────────────────────────
    float flicker = 1.0 - flicker_amount * abs(sin(TIME * flicker_speed + 1.37));

    // ── Pulse (synced with chest core OmniLight) ─────────────────────────────
    float pulse   = 1.0 + pulse_amount * sin(TIME * pulse_speed);

    // ── Compose emission ────────────────────────────────────────────────────
    vec3 base_emit  = emission_color.rgb * (0.6 + scanline * 0.4 + band);
    vec3 edge_emit  = edge_color.rgb * fresnel * 1.6;
    vec3 scan_emit  = scanline_color.rgb * scanline * 0.5;
    vec3 final_emit = (base_emit + edge_emit + scan_emit) * flicker * pulse;

    ALBEDO    = final_emit;
    EMISSION  = final_emit;

    // ── Alpha: stronger at edges, modulated by scanlines ────────────────────
    float alpha = base_opacity * (0.4 + fresnel * 0.6 + scanline * 0.3 + band);
    ALPHA = clamp(alpha * flicker, 0.0, 0.95);
}
