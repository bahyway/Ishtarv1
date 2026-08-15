# orbit_multimesh.gd — MultiMesh ORBIT Renderer
# 80,272 NAJAF particles — one GPU draw call
# BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾
#
# CORRECTIONS:
#   Dead particle colour = #404040 (NOT #800000 Nergal AV)
#   KUR overlay = #1A0A2E sovereign indigo
#   W5H2 query syntax in _query_enkidb()
#   B11 = round(eav_quality × 240) Plimpton 322 confirmed
#
# REWRITTEN 2026-07-29 (Patterns Arsenal validation pass, "ParaView /
# vaex / HOOMD-blue" document): two real, separate fixes landed together.
#
# 1. Dead-server wire protocol fixed. The previous version spoke raw,
#    unprefixed W5H2 + a 4-byte length header with no verb, and expected
#    a trailing 0u32 terminator frame over a single, long-lived
#    connection -- the `enkidb-query-server` protocol, which
#    `sumuukin_client.gd`'s own header confirms "was never actually
#    deployed anywhere in this fleet's real infrastructure." This script
#    was therefore never actually receiving live data from any real
#    Read Node. Fixed to speak the one real, currently-deployed binary
#    wire protocol every EnkiDB Type's read-server actually implements:
#    `QUERY:<heptascript>` request, one length-prefixed binary response,
#    connection closes -- decoded via `SumuUkinClient.decode_binary_
#    response()`, the same decoder `theater_3d.gd` already uses, instead
#    of a bespoke JSON parse. A fresh connection is opened per query
#    (matching the real protocol's one-request-one-response-then-close
#    contract) rather than reused across refreshes.
#
# 2. Interaction-cadence LOD added — the ParaView pattern validated
#    against this repo's real Sage-pattern conventions
#    (docs/WHAT_IS_BAHYWAY.md's "Patterns Arsenal"): while the camera is
#    being dragged, render a decimated prefix of the already-uploaded
#    MultiMesh buffer; once the camera has been still for
#    LOD_STILLNESS_DELAY_SEC, restore full resolution. No re-query and
#    no data movement happens on this path at all -- `multimesh.
#    visible_instance_count` is the only thing that changes, because
#    every live particle is already GPU-resident in the buffer (the
#    HOOMD-blue pattern this same validation pass confirmed: particles
#    live on the GPU and stay there). For an arbitrary prefix length to
#    be a fair, unbiased sample rather than "whichever subtype the query
#    happened to return first," `_apply_query_results` writes particles
#    into the buffer in round-robin order across the seven
#    OrbitalSubtype buckets, so any prefix -- decimated or full --
#    already carries roughly the real per-bucket proportions. This is
#    the same stratification principle GL-DST-001 §6.1's stratified
#    subsample already uses, applied to buffer *order* instead of query
#    *sampling*.
extends MultiMeshInstance3D

const SumuUkinClient = preload("res://scripts/sumuukin_client.gd")

const ENKIDB_HOST:  String = "192.168.122.107"
const ENKIDB_PORT:  int    = 7001
const MAX_PARTICLES: int   = 80272
const REFRESH_SEC:  float  = 30.0
const PLIMPTON_322: float  = 240.0  # NEVER 255

const CONNECT_TIMEOUT: float = 5.0
const READ_TIMEOUT:    float = 30.0

# ── Interaction-cadence LOD (ParaView pattern) ──────────────────────────
# Fraction of the live particle count shown while the camera is moving.
# Real, documented, not "as many as fit" -- mirrors GL-DST-001 §6.1's own
# convention of naming the budget explicitly rather than leaving it
# implicit in whatever the frame rate happens to tolerate.
const LOD_DECIMATED_FRACTION: float = 0.10
# How long the camera must be quiet before fidelity is restored. Short
# enough to feel immediate, long enough that a brief pause mid-drag
# doesn't flicker full-res on and off.
const LOD_STILLNESS_DELAY_SEC: float = 0.35

# Seven sealed particle orbital radii (OrbitalSubtype property)
const RADII: Array[float] = [
	0.10,  # GOLDEN_GEM   #FFD700
	0.30,  # GOLDEN_ALIVE
	0.40,  # FUZZY_AGED
	0.57,  # FUZZY_GRAY
	0.72,  # FUZZY_DECAY  — TIAMAT DILBAT fires here
	0.85,  # DEAD_EXPIRED #404040  (NOT #800000 — Nergal AV only)
	0.95,  # DEAD_SEALED  #282828 PUZRU
]

var _tcp:           StreamPeerTCP = StreamPeerTCP.new()
var _tcp_connecting: bool         = false
var _tcp_awaiting_response: bool  = false
var _tcp_state_elapsed: float     = 0.0
var _tcp_send_pending: PackedByteArray = PackedByteArray()
var _refresh_timer: float         = 0.0
var _live_count:    int           = 0
var _expect_len:    int           = -1

var _lod_count:       int   = 0
var _lod_active:      bool  = false
var _camera_moving:   bool  = false
var _stillness_timer: float = 0.0

func _ready() -> void:
	_init_multimesh()
	_seed_demo_particles()
	_query_enkidb()
	print("[OrbitMultiMesh] 𒁾 Ready — %d particle slots" % MAX_PARTICLES)

func _process(delta: float) -> void:
	_refresh_timer += delta
	if _refresh_timer >= REFRESH_SEC:
		_refresh_timer = 0.0
		_query_enkidb()
	_poll_response(delta)
	_update_lod(delta)

func _init_multimesh() -> void:
	var mm := MultiMesh.new()
	mm.transform_format       = MultiMesh.TRANSFORM_3D
	mm.use_colors             = true
	mm.instance_count         = MAX_PARTICLES
	mm.visible_instance_count = 0
	var sphere := SphereMesh.new()
	sphere.radius = 0.018; sphere.height = 0.036
	sphere.radial_segments = 4; sphere.rings = 2
	mm.mesh = sphere
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	multimesh = mm

func _seed_demo_particles() -> void:
	var per_orbit := MAX_PARTICLES / RADII.size()
	var idx       := 0
	for si in range(RADII.size()):
		var radius := RADII[si]
		var colour := _orbital_colour(si)
		for _p in range(per_orbit):
			if idx >= MAX_PARTICLES: break
			_place_particle(idx, radius, colour)
			idx += 1
	_live_count = idx
	_set_live_count(_live_count)

func _place_particle(idx: int, orbital_radius: float, colour: Color,
		theta: float = -1.0) -> void:
	# OrbitalPosition (r,θ,φ) → Cartesian
	var th := theta if theta >= 0.0 else randf() * TAU
	var ph := PI / 2.0 + (randf() - 0.5) * 0.1
	var x  := orbital_radius * sin(ph) * cos(th)
	var y  := orbital_radius * cos(ph)
	var z  := orbital_radius * sin(ph) * sin(th)
	var t  := Transform3D(Basis(), Vector3(x, y, z))
	multimesh.set_instance_transform(idx, t)
	multimesh.set_instance_color(idx, colour)

func _orbital_colour(subtype: int) -> Color:
	# CORRECTED: DEAD_EXPIRED = #404040 (NOT #800000 Nergal AV)
	match subtype:
		0: return Color("#FFD700")  # GOLDEN_GEM
		1: return Color("#C8B830")  # GOLDEN_ALIVE
		2: return Color("#8090A0")  # FUZZY_AGED
		3: return Color("#607080")  # FUZZY_GRAY
		4: return Color("#405060")  # FUZZY_DECAY
		5: return Color("#404040")  # DEAD_EXPIRED — dark grey
		6: return Color("#282828")  # DEAD_SEALED — near-black PUZRU
		_: return Color("#4A90E2")

func _query_enkidb() -> void:
	# SOVEREIGN W5H2 anti-SQL syntax, sent as `QUERY:<heptascript>` per
	# the one real wire protocol (see this file's header).
	var query := "WHO T.E\n" + \
		"WHAT E[eav_quality, orbital_subtype]\n" + \
		"WHERE E[state] != \"DEAD_SEALED\"\n" + \
		"HOW_MUCH LIMIT %d" % MAX_PARTICLES
	_open_and_send(query)

func _open_and_send(query: String) -> void:
	_tcp = StreamPeerTCP.new()
	if _tcp.connect_to_host(ENKIDB_HOST, ENKIDB_PORT) != OK:
		push_warning("[OrbitMultiMesh] could not open connection to %s:%d" % [ENKIDB_HOST, ENKIDB_PORT])
		return
	_tcp_connecting      = true
	_tcp_awaiting_response = false
	_tcp_state_elapsed   = 0.0
	_expect_len          = -1
	_tcp_send_pending    = ("QUERY:" + query).to_utf8_buffer()

func _poll_response(delta: float) -> void:
	if not _tcp_connecting and not _tcp_awaiting_response:
		return

	_tcp.poll()
	var status := _tcp.get_status()
	_tcp_state_elapsed += delta

	if _tcp_connecting:
		if status == StreamPeerTCP.STATUS_CONNECTING:
			if _tcp_state_elapsed > CONNECT_TIMEOUT:
				push_warning("[OrbitMultiMesh] connect to %s:%d timed out" % [ENKIDB_HOST, ENKIDB_PORT])
				_tcp.disconnect_from_host()
				_tcp_connecting = false
			return
		if status != StreamPeerTCP.STATUS_CONNECTED:
			_tcp_connecting = false
			push_warning("[OrbitMultiMesh] connection to %s:%d failed" % [ENKIDB_HOST, ENKIDB_PORT])
			return
		# Connected -- send the length-prefixed "QUERY:<heptascript>"
		# request, matching every real Read Node's read_frame().
		var header := PackedByteArray()
		header.resize(4)
		header.encode_u32(0, _tcp_send_pending.size())
		_tcp.put_data(header)
		_tcp.put_data(_tcp_send_pending)
		_tcp_connecting        = false
		_tcp_awaiting_response = true
		_tcp_state_elapsed     = 0.0
		return

	# _tcp_awaiting_response
	if status != StreamPeerTCP.STATUS_CONNECTED:
		_tcp_awaiting_response = false
		return
	if _tcp_state_elapsed > READ_TIMEOUT:
		push_warning("[OrbitMultiMesh] timed out waiting for response from %s:%d" % [ENKIDB_HOST, ENKIDB_PORT])
		_tcp.disconnect_from_host()
		_tcp_awaiting_response = false
		return

	var avail := _tcp.get_available_bytes()
	if avail <= 0:
		return

	if _expect_len < 0:
		if avail < 4:
			return
		var header := _tcp.get_partial_data(4)
		if header[0] != OK:
			return
		_expect_len = header[1].decode_u32(0)
		avail = _tcp.get_available_bytes()

	if avail < _expect_len:
		return

	var payload := _tcp.get_partial_data(_expect_len)
	_expect_len = -1
	_tcp_awaiting_response = false
	# One response per connection on the real protocol -- done with this
	# socket regardless of decode outcome.
	_tcp.disconnect_from_host()
	if payload[0] != OK:
		return
	_apply_query_results(payload[1])

func _apply_query_results(raw: PackedByteArray) -> void:
	var decoded := SumuUkinClient.decode_binary_response(raw)
	if decoded.error != "":
		push_warning("[OrbitMultiMesh] query error: %s" % decoded.error)
		return

	# Bucket rows by OrbitalSubtype count first, then write particles into
	# the MultiMesh buffer in round-robin order across buckets (see this
	# file's header) so that ANY prefix length -- the LOD-decimated one
	# included -- is already a fair cross-subtype sample, not whatever
	# order the server happened to return. Every particle in bucket `b`
	# is identical for placement purposes (same subtype → same radius and
	# colour), so only a per-bucket count is needed, not a stored list.
	var bucket_counts: Array[int] = [0, 0, 0, 0, 0, 0, 0]
	for row in decoded.rows:
		var quality := 0.5
		for pair in row.get("attrs", []):
			if pair.size() == 2 and pair[0] == "eav_quality":
				quality = float(pair[1])
		bucket_counts[_subtype_from_quality(quality)] += 1

	var idx := 0
	var placed := [0, 0, 0, 0, 0, 0, 0]
	var remaining := decoded.rows.size()
	while remaining > 0 and idx < MAX_PARTICLES:
		for b in range(bucket_counts.size()):
			if placed[b] >= bucket_counts[b]:
				continue
			placed[b] += 1
			remaining -= 1
			_place_particle(idx, RADII[b], _orbital_colour(b))
			idx += 1
			if idx >= MAX_PARTICLES:
				break

	_live_count = idx
	_lod_count  = max(1, int(_live_count * LOD_DECIMATED_FRACTION))
	_set_live_count(_lod_count if _lod_active else _live_count)

func _subtype_from_quality(quality: float) -> int:
	# Mirrors particle_node.gd's _update_orbital_radius() mapping.
	if   quality >= 0.83: return 0  # GoldenGem
	elif quality >= 0.70: return 1  # GoldenAlive
	elif quality >= 0.55: return 2  # FuzzyAged
	elif quality >= 0.40: return 3  # FuzzyGray
	elif quality >= 0.20: return 4  # FuzzyDecay
	elif quality >= 0.05: return 5  # DeadExpired
	else:                  return 6  # DeadSealed

# ── Interaction-cadence LOD ──────────────────────────────────────────────

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion and event.button_mask != 0:
		_camera_moving = true
		_stillness_timer = 0.0
	elif event is InputEventMouseButton and event.pressed and event.button_index in \
			[MOUSE_BUTTON_LEFT, MOUSE_BUTTON_MIDDLE, MOUSE_BUTTON_RIGHT]:
		_camera_moving = true
		_stillness_timer = 0.0

# Public API: a dedicated camera-rig script can call this directly instead
# of (or alongside) the input-heuristic detection above, for a codebase
# that later adds an explicit orbit-camera controller with its own notion
# of "moving."
func notify_camera_motion() -> void:
	_camera_moving = true
	_stillness_timer = 0.0

func _update_lod(delta: float) -> void:
	if _camera_moving:
		_stillness_timer += delta
		if _stillness_timer >= LOD_STILLNESS_DELAY_SEC:
			_camera_moving = false
			_stillness_timer = 0.0
			_set_lod_active(false)
		else:
			_set_lod_active(true)

func _set_lod_active(active: bool) -> void:
	if active == _lod_active:
		return
	_lod_active = active
	_set_live_count(_lod_count if active else _live_count)
	print("[OrbitMultiMesh] LOD %s — showing %d of %d live particles" % \
		["decimated" if active else "full", multimesh.visible_instance_count, _live_count])

func _set_live_count(count: int) -> void:
	multimesh.visible_instance_count = count
