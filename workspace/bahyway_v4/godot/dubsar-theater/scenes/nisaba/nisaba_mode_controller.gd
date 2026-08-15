# ══════════════════════════════════════════════════════════════════════════════
# nisaba_mode_controller.gd
# NISABA 𒀭𒉀 — Seven-Mode State Machine
# BahyWay.Ecosystem v4.0 — DUB.SAR Bahaa Fadam
#
# Controls NISABA's seven modes by driving:
#   - Hologram shader uniforms (colour, pulse, flicker)
#   - Crown rotation speed and colour
#   - Eye glow intensity
#   - Chest core light energy
#   - Particle field behaviour
#   - Whisper label text
#   - Animation playback
#
# MODES:
#   1 — IDLE            Crown slow, cyan. Watching always.
#   2 — PATTERN_DISCOVERY  Crown fast, white sparks. New pattern found.
#   3 — PROPOSAL        Crown medium, gold. Awaiting DUB.SAR decision.
#   4 — TEACHING        Crown medium, maroon. Sending to KIBRATU.
#   5 — SEALING         Crown gold flash. UTNAPISHTIM fires.
#   6 — WITNESS         Crown still. Pattern rejected / PUZRU sealed.
#   7 — TUPSHARU        Crown lapis. Mode 7 art plate generating.
# ══════════════════════════════════════════════════════════════════════════════

extends Node3D

# ── Node references (set in scene) ──────────────────────────────────────────
@export var body_mesh       : MeshInstance3D
@export var head_mesh       : MeshInstance3D
@export var crown_pivot     : Node3D
@export var crown_ring      : MeshInstance3D
@export var crown_particles : GPUParticles3D
@export var chest_light     : OmniLight3D
@export var left_eye_light  : OmniLight3D
@export var right_eye_light : OmniLight3D
@export var platform_particles : GPUParticles3D
@export var particle_field  : GPUParticles3D
@export var whisper_label   : Label3D
@export var animation_player : AnimationPlayer

# ── Mode colours (sealed) ───────────────────────────────────────────────────
const MODE_COLOURS = {
	1: { # IDLE — cyan
		"emission": Color(0.0,  0.82, 1.0,  1.0),
		"edge":     Color(0.25, 0.95, 1.0,  1.0),
		"crown":    Color(0.0,  0.82, 1.0,  1.0),
		"chest":    Color(0.0,  0.85, 1.0),
		"eye":      Color(0.0,  1.0,  1.0),
	},
	2: { # PATTERN_DISCOVERY — white sparks
		"emission": Color(0.9,  0.95, 1.0,  1.0),
		"edge":     Color(1.0,  1.0,  1.0,  1.0),
		"crown":    Color(1.0,  1.0,  1.0,  1.0),
		"chest":    Color(0.8,  0.9,  1.0),
		"eye":      Color(1.0,  1.0,  1.0),
	},
	3: { # PROPOSAL — gold
		"emission": Color(0.0,  0.82, 1.0,  1.0),
		"edge":     Color(0.78, 0.66, 0.29, 1.0),
		"crown":    Color(0.78, 0.66, 0.29, 1.0),  # #C8A84B
		"chest":    Color(0.78, 0.66, 0.29),
		"eye":      Color(0.0,  1.0,  1.0),
	},
	4: { # TEACHING — maroon connection
		"emission": Color(0.0,  0.82, 1.0,  1.0),
		"edge":     Color(0.5,  0.0,  0.0,  1.0),
		"crown":    Color(0.5,  0.0,  0.0,  1.0),  # #800000
		"chest":    Color(0.8,  0.1,  0.1),
		"eye":      Color(0.8,  0.2,  0.2),
	},
	5: { # SEALING — gold flash
		"emission": Color(0.78, 0.66, 0.29, 1.0),
		"edge":     Color(1.0,  0.9,  0.5,  1.0),
		"crown":    Color(1.0,  0.9,  0.4,  1.0),
		"chest":    Color(1.0,  0.85, 0.3),
		"eye":      Color(1.0,  0.9,  0.3),
	},
	6: { # WITNESS — still, dim
		"emission": Color(0.0,  0.5,  0.7,  0.6),
		"edge":     Color(0.0,  0.6,  0.8,  0.5),
		"crown":    Color(0.0,  0.5,  0.7,  0.5),
		"chest":    Color(0.0,  0.5,  0.7),
		"eye":      Color(0.0,  0.6,  0.8),
	},
	7: { # TUPSHARU — lapis lazuli
		"emission": Color(0.22, 0.42, 0.78, 1.0),
		"edge":     Color(0.4,  0.6,  1.0,  1.0),
		"crown":    Color(0.78, 0.66, 0.29, 1.0),  # gold crown always for art
		"chest":    Color(0.3,  0.5,  1.0),
		"eye":      Color(0.5,  0.7,  1.0),
	},
}

const MODE_CROWN_SPEED = {
	1: 0.8,   # slow — idle
	2: 4.0,   # fast — discovery
	3: 2.0,   # medium — proposal
	4: 2.2,   # medium — teaching
	5: 8.0,   # very fast — sealing flash
	6: 0.0,   # stopped — witness
	7: 1.5,   # steady — scribal
}

const MODE_WHISPERS = {
	1: "𒀭𒉀 watching...",
	2: "𒀭𒉀 NEW PATTERN FOUND",
	3: "𒀭𒉀 DUB.SAR — your decision",
	4: "𒀭𒉀 teaching KIBRATU...",
	5: "𒀭𒉀 UTNAPISHTIM seals",
	6: "𒀭𒉀 witnessed",
	7: "𒀭𒉀 ṬUPŠARRU drawing...",
}

const MODE_CHEST_ENERGY = {
	1: 1.2,
	2: 2.5,
	3: 2.0,
	4: 1.8,
	5: 4.0,
	6: 0.6,
	7: 3.0,
}

const MODE_PARTICLE_RATE = {
	1: 4,
	2: 80,
	3: 20,
	4: 15,
	5: 120,
	6: 2,
	7: 40,
}

# ── State ────────────────────────────────────────────────────────────────────
var current_mode  : int  = 1
var target_mode   : int  = 1
var transition_t  : float = 0.0
var crown_angle   : float = 0.0
var _sealing_flash_timer : float = 0.0

# Materials (cached)
var _body_mat   : ShaderMaterial
var _crown_mat  : ShaderMaterial

# ── Signals ──────────────────────────────────────────────────────────────────
signal mode_changed(new_mode: int, mode_name: String)
signal pattern_proposed(pattern_name: String)
signal template_sealed(pattern_name: String)

# ── Lifecycle ────────────────────────────────────────────────────────────────
func _ready() -> void:
	_cache_materials()
	set_mode(1)
	print("[NISABA] 𒀭𒉀 Sovereign Hologram Agent v4.0 — online")
	print("[NISABA] Seven modes available. Mode 1: IDLE.")

func _process(delta: float) -> void:
	_rotate_crown(delta)
	_update_transition(delta)
	_handle_sealing_flash(delta)

# ── Public API ───────────────────────────────────────────────────────────────

## Set NISABA to a specific mode (1–7)
## Called by BeeMDM pipeline, KIBRATU, or DUB.SAR UI
func set_mode(mode: int, instant: bool = false) -> void:
	if mode < 1 or mode > 7:
		push_warning("[NISABA] Invalid mode: %d" % mode)
		return

	var prev_mode = current_mode
	target_mode   = mode

	if instant:
		current_mode = mode
		_apply_mode(mode)
	else:
		transition_t = 0.0

	# Trigger sealing flash
	if mode == 5:
		_sealing_flash_timer = 0.0

	# Update whisper
	if whisper_label:
		whisper_label.text = MODE_WHISPERS.get(mode, "")

	# Update particle emission rate
	if crown_particles:
		crown_particles.amount = MODE_PARTICLE_RATE.get(mode, 10)

	if platform_particles:
		platform_particles.emitting = (mode != 6)

	# Play animation if exists
	if animation_player:
		var anim_name = _mode_animation_name(mode)
		if animation_player.has_animation(anim_name):
			animation_player.play(anim_name)

	emit_signal("mode_changed", mode, _mode_name(mode))
	print("[NISABA] Mode %d → %s" % [mode, _mode_name(mode)])

## Called by NISABA orchestrator when pattern found (Mode 2)
func on_pattern_discovered(pattern_name: String) -> void:
	set_mode(2)
	if whisper_label:
		whisper_label.text = "𒀭𒉀 FOUND: %s" % pattern_name
	print("[NISABA] Pattern discovered: %s" % pattern_name)

## Called when presenting proposal to DUB.SAR (Mode 3)
func on_proposal_ready(pattern_name: String) -> void:
	set_mode(3)
	if whisper_label:
		whisper_label.text = "𒀭𒉀 %s — seal it?" % pattern_name
	emit_signal("pattern_proposed", pattern_name)

## Called when DUB.SAR approves (Mode 5 flash → back to 1)
func on_dubsar_approved(pattern_name: String) -> void:
	set_mode(5)
	if whisper_label:
		whisper_label.text = "𒀭𒉀 %s — sealed 𒌓𒍣𒅁𒀭" % pattern_name
	emit_signal("template_sealed", pattern_name)
	# Return to idle after flash
	await get_tree().create_timer(1.2).timeout
	set_mode(1)

## Called when DUB.SAR rejects (Mode 6)
func on_dubsar_rejected(pattern_name: String, permanent: bool) -> void:
	set_mode(6)
	var msg = "𒀭𒉀 %s — never again" if permanent else "𒀭𒉀 %s — watching"
	if whisper_label:
		whisper_label.text = msg % pattern_name
	await get_tree().create_timer(2.0).timeout
	set_mode(1)

## Called when KIBRATU finds discovery (Mode 7 — TUPSHARU)
func on_tupsharu_triggered(plate_name: String) -> void:
	set_mode(7)
	if whisper_label:
		whisper_label.text = "𒀭𒉀 drawing %s..." % plate_name

# ── Internal ──────────────────────────────────────────────────────────────────

func _cache_materials() -> void:
	if body_mesh and body_mesh.get_surface_override_material_count() > 0:
		_body_mat = body_mesh.get_active_material(0) as ShaderMaterial
	if crown_ring and crown_ring.get_surface_override_material_count() > 0:
		_crown_mat = crown_ring.get_active_material(0) as ShaderMaterial

func _apply_mode(mode: int) -> void:
	var c = MODE_COLOURS.get(mode, MODE_COLOURS[1])

	# Body shader uniforms
	if _body_mat:
		_body_mat.set_shader_parameter("emission_color", c["emission"])
		_body_mat.set_shader_parameter("edge_color",     c["edge"])
		_body_mat.set_shader_parameter("scanline_speed", _scanline_speed_for_mode(mode))
		_body_mat.set_shader_parameter("pulse_speed",    _pulse_speed_for_mode(mode))
		_body_mat.set_shader_parameter("flicker_amount", _flicker_for_mode(mode))

	# Crown colour
	if _crown_mat:
		_crown_mat.set_shader_parameter("emission_color", Color(c["crown"]))

	# Chest light
	if chest_light:
		chest_light.light_color  = c["chest"]
		chest_light.light_energy = MODE_CHEST_ENERGY.get(mode, 1.2)

	# Eye lights
	if left_eye_light:
		left_eye_light.light_color  = c["eye"]
		left_eye_light.light_energy = 1.8 if mode != 6 else 0.4
	if right_eye_light:
		right_eye_light.light_color  = c["eye"]
		right_eye_light.light_energy = 1.8 if mode != 6 else 0.4

func _rotate_crown(delta: float) -> void:
	if not crown_pivot:
		return
	var speed = MODE_CROWN_SPEED.get(target_mode, 0.8)
	crown_angle += speed * delta
	crown_pivot.rotation.y = crown_angle

func _update_transition(delta: float) -> void:
	if current_mode == target_mode:
		return
	transition_t += delta * 2.5  # transition speed
	if transition_t >= 1.0:
		transition_t = 1.0
		current_mode = target_mode
	_apply_mode_lerp(current_mode, target_mode, transition_t)

func _apply_mode_lerp(from_mode: int, to_mode: int, t: float) -> void:
	var c_from = MODE_COLOURS.get(from_mode, MODE_COLOURS[1])
	var c_to   = MODE_COLOURS.get(to_mode,   MODE_COLOURS[1])

	if _body_mat:
		_body_mat.set_shader_parameter("emission_color",
			c_from["emission"].lerp(c_to["emission"], t))
		_body_mat.set_shader_parameter("edge_color",
			c_from["edge"].lerp(c_to["edge"], t))

	if chest_light:
		chest_light.light_color = c_from["chest"].lerp(c_to["chest"], t)
		chest_light.light_energy = lerp(
			float(MODE_CHEST_ENERGY.get(from_mode, 1.2)),
			float(MODE_CHEST_ENERGY.get(to_mode,   1.2)),
			t)

func _handle_sealing_flash(delta: float) -> void:
	if current_mode != 5:
		return
	_sealing_flash_timer += delta
	# Gold flash pulse
	if _body_mat:
		var flash = abs(sin(_sealing_flash_timer * 12.0))
		_body_mat.set_shader_parameter("base_opacity", 0.6 + flash * 0.35)
	if chest_light:
		chest_light.light_energy = 2.0 + abs(sin(_sealing_flash_timer * 10.0)) * 3.0

func _scanline_speed_for_mode(mode: int) -> float:
	match mode:
		1: return 1.6
		2: return 4.0
		3: return 2.0
		4: return 2.2
		5: return 6.0
		6: return 0.8
		7: return 2.5
	return 1.6

func _pulse_speed_for_mode(mode: int) -> float:
	match mode:
		2: return 4.0
		5: return 8.0
		7: return 3.0
		_: return 1.8

func _flicker_for_mode(mode: int) -> float:
	match mode:
		2: return 0.12  # more flicker on discovery
		5: return 0.0   # no flicker during seal flash (clean gold)
		6: return 0.08
		_: return 0.06

func _mode_animation_name(mode: int) -> String:
	match mode:
		1: return "nisaba_idle"
		2: return "nisaba_discovery"
		3: return "nisaba_proposal"
		4: return "nisaba_teaching"
		5: return "nisaba_sealing"
		6: return "nisaba_witness"
		7: return "nisaba_tupsharu"
	return "nisaba_idle"

func _mode_name(mode: int) -> String:
	match mode:
		1: return "IDLE"
		2: return "PATTERN_DISCOVERY"
		3: return "PROPOSAL"
		4: return "TEACHING"
		5: return "SEALING"
		6: return "WITNESS"
		7: return "TUPSHARU"
	return "UNKNOWN"
