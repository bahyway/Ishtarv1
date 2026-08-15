# bigring.gd — BIGRING Sovereign Particle Ring
# BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾
#
# CORRECTIONS:
#   Dead particle colour = #404040 dark grey (NOT #800000 Nergal AV)
#   KUR terminal overlay = #1A0A2E sovereign indigo (was absent)
#   TIAMAT set_tiamat_alert() now handles all 4 levels including KUR
#   B11 = round(eav_quality × 240) Plimpton 322 — confirmed correct
extends GPUParticles3D

# ── Sovereign colour constants (CORRECTED) ─────────────────────
const COLOR_GOLDEN_GEM = Color("#FFD700")   # pure gold
const COLOR_KAKKAB     = Color("#D46000")   # orange
const COLOR_ERRA       = Color("#CC2244")   # crimson
const COLOR_KUR        = Color("#1A0A2E")   # sovereign indigo ← CORRECTED
const COLOR_DILBAT     = Color("#E8B400")   # amber
const COLOR_FUZZY      = Color("#4A90E2")   # blue
# CORRECTED: DEAD = #404040 dark grey (NOT #800000 — that is Nergal AV)
const COLOR_DEAD       = Color("#404040")
# NERGAL AV = #800000 — belongs to AV engine ONLY — never here
const COLOR_GEM        = Color("#FFD700")

const PLIMPTON_322: float = 240.0  # B11 divisor — NEVER 255

var _tiamat_level:  int   = 0
var _flash_active:  bool  = false
var _flash_timer:   float = 0.0
var _base_emission: float = 1.0

func _ready() -> void:
	process_material = _build_particle_material()
	emitting          = true
	print("[BIGRING] 𒁾 Ready — 80,272 particle orbit")

func _process(delta: float) -> void:
	if _flash_active:
		_flash_timer -= delta
		if _flash_timer <= 0.0:
			_flash_active = false
			_set_ring_colour(COLOR_GOLDEN_GEM)

func on_utnapishtim_seal() -> void:
	_set_ring_colour(COLOR_GOLDEN_GEM)
	_flash_active = true
	_flash_timer  = 2.0
	print("[BIGRING] 𒌓𒍣𒅁𒀭 UTNAPISHTIM seal — flashing gold")

func set_emission_intensity(intensity: float) -> void:
	_base_emission = clamp(intensity, 0.1, 2.0)
	if process_material:
		process_material.emission_energy_multiplier = _base_emission

func set_tiamat_alert(level: int) -> void:
	# CORRECTED: all 4 TIAMAT levels handled (KUR was missing)
	_tiamat_level = level
	match level:
		0: _restore_nominal_colour()
		1: _set_ring_colour(COLOR_DILBAT)
		2: _set_ring_colour(COLOR_KAKKAB)
		3: _set_ring_colour(COLOR_ERRA)
		4: _set_ring_colour(COLOR_KUR)    # KUR 𒆳 — CORRECTED: was missing
	if level > 0:
		print("[BIGRING] TIAMAT %s" % _tiamat_label(level))

func _tiamat_label(level: int) -> String:
	match level:
		1: return "DILBAT 𒀭𒁾𒁍"
		2: return "KAKKAB 𒀭𒆳𒀝"
		3: return "ERRA 𒀭𒂗𒆳"
		4: return "KUR 𒆳"      # CORRECTED: was missing
		_: return "NOMINAL"

func _build_particle_material() -> ParticleProcessMaterial:
	var mat := ParticleProcessMaterial.new()
	mat.emission_shape             = ParticleProcessMaterial.EMISSION_SHAPE_RING
	mat.emission_ring_radius       = 4.0
	mat.emission_ring_inner_radius = 0.5
	mat.emission_ring_height       = 0.1
	mat.emission_ring_axis         = Vector3(0, 1, 0)
	mat.gravity                    = Vector3.ZERO
	mat.initial_velocity_min       = 0.05
	mat.initial_velocity_max       = 0.18
	mat.color                      = COLOR_GOLDEN_GEM
	mat.color_ramp                 = _build_sovereign_gradient()
	mat.scale_min                  = 0.008
	mat.scale_max                  = 0.022
	return mat

func _build_sovereign_gradient() -> Gradient:
	# CORRECTED gradient: #404040 for dead, #1A0A2E for KUR
	# NOT #800000 Maroon (that is Nergal AV engine only)
	var g := Gradient.new()
	g.set_color(0, COLOR_GOLDEN_GEM)        # birth: GOLDEN_GEM
	g.add_point(0.40, Color("#4A90E2"))      # FUZZY zone: blue
	g.add_point(0.72, Color("#808080"))      # FUZZY_DECAY: grey
	g.add_point(0.85, COLOR_DEAD)            # DEAD_EXPIRED: #404040
	g.set_color(1, Color(0.10, 0.04, 0.18, 0)) # DEAD_SEALED: #1A0A2E fade
	return g

func _set_ring_colour(colour: Color) -> void:
	if process_material: process_material.color = colour

func _restore_nominal_colour() -> void:
	_set_ring_colour(COLOR_GOLDEN_GEM)
	_base_emission = 1.0
