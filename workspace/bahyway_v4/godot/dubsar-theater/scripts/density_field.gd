# density_field.gd — Exact-Aggregation Density Field (the "vaex pattern")
# BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾
#
# Built 2026-07-29, from validating a shared document on ParaView/vaex/
# HOOMD-blue's billion-element visualization patterns against this repo's
# real Sage-pattern conventions (docs/WHAT_IS_BAHYWAY.md's "Patterns
# Arsenal"). vaex's own honesty about what "visualizing a billion" means
# is the pattern here: it never draws individual points at all -- it
# renders a binned density image where every pixel is an EXACT aggregate
# of millions of rows, computed in well under a second. GL-DST-001 §6.1's
# "Statistical Witnessing" already gives DubSar's ORBIT 3D lens a
# stratified *sample* fallback at overview scale; this script is the
# complementary, distinct alternative: an EXACT aggregate, computed
# server-side by HeptaScript's real GRAVITY/MEASURE clauses (no sampling
# anywhere in the pipeline), rendered as one glyph per band instead of one
# glyph per particle.
#
# Simplification stated plainly: vaex's actual output is a 2D binned
# heatmap image. This renders one scaled 3D marker per GRAVITY band
# instead -- reusing this scene's existing sphere/radius visual language
# (see orbit_multimesh.gd) rather than adding a texture/shader pipeline.
# The *aggregation* is the real, exact vaex pattern; the *glyph* is a
# BahyWay-native rendering choice, not a claim of building vaex's heatmap.
#
# Query: `GRAVITY E[eav_quality] BAND <width> MEASURE DENSE` -- BAND
# groups the exact-value attribute into fixed-width buckets (parser.rs's
# own safety rule: GRAVITY always requires BAND or MAX_GROUPS, so this
# can never grow unbounded regardless of corpus size), and MEASURE DENSE
# is a real O(N) exact count per band, not an estimate.
extends MultiMeshInstance3D

const SumuUkinClient = preload("res://scripts/sumuukin_client.gd")

const ENKIDB_HOST:   String = "192.168.122.107"
const ENKIDB_PORT:   int    = 7001
const REFRESH_SEC:   float  = 10.0
const CONNECT_TIMEOUT: float = 5.0
const READ_TIMEOUT:    float = 30.0

# Band width over eav_quality's real [0.0, 1.0] domain -- 0.02 gives up to
# 50 bands, comfortably bounded regardless of how many billion particles
# actually exist, since GRAVITY's own cost is O(bands), not O(particles),
# once the exact count is computed server-side.
const BAND_WIDTH: float = 0.02
const MAX_BANDS:  int   = 64

# Same seven OrbitalSubtype radii/colours orbit_multimesh.gd uses, so the
# density field and the particle field share one visual language.
const RADII: Array[float] = [0.10, 0.30, 0.40, 0.57, 0.72, 0.85, 0.95]

var _tcp: StreamPeerTCP = StreamPeerTCP.new()
var _tcp_connecting: bool = false
var _tcp_awaiting_response: bool = false
var _tcp_state_elapsed: float = 0.0
var _tcp_send_pending: PackedByteArray = PackedByteArray()
var _expect_len: int = -1
var _refresh_timer: float = 0.0
var _live_count: int = 0

func _ready() -> void:
	_init_multimesh()
	_query_density()
	print("[DensityField] 𒁾 Ready — exact-aggregation density field, %d band slots" % MAX_BANDS)

func _process(delta: float) -> void:
	_refresh_timer += delta
	if _refresh_timer >= REFRESH_SEC:
		_refresh_timer = 0.0
		_query_density()
	_poll_response(delta)

func _init_multimesh() -> void:
	var mm := MultiMesh.new()
	mm.transform_format       = MultiMesh.TRANSFORM_3D
	mm.use_colors             = true
	mm.instance_count         = MAX_BANDS
	mm.visible_instance_count = 0
	var sphere := SphereMesh.new()
	sphere.radius = 0.03; sphere.height = 0.06
	sphere.radial_segments = 8; sphere.rings = 4
	mm.mesh = sphere
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	multimesh = mm

func _orbital_colour(subtype: int) -> Color:
	match subtype:
		0: return Color("#FFD700")  # GOLDEN_GEM
		1: return Color("#C8B830")  # GOLDEN_ALIVE
		2: return Color("#8090A0")  # FUZZY_AGED
		3: return Color("#607080")  # FUZZY_GRAY
		4: return Color("#405060")  # FUZZY_DECAY
		5: return Color("#404040")  # DEAD_EXPIRED
		6: return Color("#282828")  # DEAD_SEALED
		_: return Color("#4A90E2")

func _subtype_from_quality(quality: float) -> int:
	if   quality >= 0.83: return 0
	elif quality >= 0.70: return 1
	elif quality >= 0.55: return 2
	elif quality >= 0.40: return 3
	elif quality >= 0.20: return 4
	elif quality >= 0.05: return 5
	else:                  return 6

func _query_density() -> void:
	var query := "WHO T.E\n" + \
		"GRAVITY E[eav_quality] BAND %s\n" % BAND_WIDTH + \
		"MEASURE DENSE"
	_open_and_send(query)

func _open_and_send(query: String) -> void:
	_tcp = StreamPeerTCP.new()
	if _tcp.connect_to_host(ENKIDB_HOST, ENKIDB_PORT) != OK:
		push_warning("[DensityField] could not open connection to %s:%d" % [ENKIDB_HOST, ENKIDB_PORT])
		return
	_tcp_connecting        = true
	_tcp_awaiting_response = false
	_tcp_state_elapsed     = 0.0
	_expect_len            = -1
	_tcp_send_pending      = ("QUERY:" + query).to_utf8_buffer()

func _poll_response(delta: float) -> void:
	if not _tcp_connecting and not _tcp_awaiting_response:
		return

	_tcp.poll()
	var status := _tcp.get_status()
	_tcp_state_elapsed += delta

	if _tcp_connecting:
		if status == StreamPeerTCP.STATUS_CONNECTING:
			if _tcp_state_elapsed > CONNECT_TIMEOUT:
				push_warning("[DensityField] connect to %s:%d timed out" % [ENKIDB_HOST, ENKIDB_PORT])
				_tcp.disconnect_from_host()
				_tcp_connecting = false
			return
		if status != StreamPeerTCP.STATUS_CONNECTED:
			_tcp_connecting = false
			push_warning("[DensityField] connection to %s:%d failed" % [ENKIDB_HOST, ENKIDB_PORT])
			return
		var header := PackedByteArray()
		header.resize(4)
		header.encode_u32(0, _tcp_send_pending.size())
		_tcp.put_data(header)
		_tcp.put_data(_tcp_send_pending)
		_tcp_connecting        = false
		_tcp_awaiting_response = true
		_tcp_state_elapsed     = 0.0
		return

	if status != StreamPeerTCP.STATUS_CONNECTED:
		_tcp_awaiting_response = false
		return
	if _tcp_state_elapsed > READ_TIMEOUT:
		push_warning("[DensityField] timed out waiting for response from %s:%d" % [ENKIDB_HOST, ENKIDB_PORT])
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
	_tcp.disconnect_from_host()
	if payload[0] != OK:
		return
	_apply_density(payload[1])

func _apply_density(raw: PackedByteArray) -> void:
	var decoded := SumuUkinClient.decode_binary_response(raw)
	if decoded.error != "":
		push_warning("[DensityField] query error: %s" % decoded.error)
		return
	var agg: Dictionary = decoded.get("aggregate", {"kind": "none"})
	if agg.kind != "grouped":
		push_warning("[DensityField] expected a grouped aggregate, got kind=%s -- is this Read Node's binary wire aggregate-trailer support current?" % agg.kind)
		return

	# Real max count across bands drives the size scale -- exact data, no
	# assumed corpus size.
	var max_count: int = 1
	for g in agg.groups:
		if int(g.count) > max_count:
			max_count = int(g.count)

	var idx := 0
	for g in agg.groups:
		if idx >= MAX_BANDS:
			break
		var key: Dictionary = g.key
		if key.tag != "band":
			continue  # Missing/Overflow buckets carry no meaningful quality position.
		var band_index: int = int(key.value)
		var quality: float = (float(band_index) + 0.5) * BAND_WIDTH
		var subtype := _subtype_from_quality(quality)
		var count: int = int(g.count)

		# Marker scale reflects the EXACT count for this band -- sqrt so a
		# 100x-denser band doesn't dwarf everything else on screen, same
		# perceptual-scaling reasoning a real heatmap's color ramp solves
		# differently (log/sqrt intensity, not linear pixel brightness).
		var scale := 0.4 + 1.6 * sqrt(float(count) / float(max_count))
		var colour := _orbital_colour(subtype)
		colour.a = clamp(0.35 + 0.65 * (float(count) / float(max_count)), 0.35, 1.0)

		var radius := RADII[subtype]
		var th := (float(band_index) / float(MAX_BANDS)) * TAU
		var x := radius * cos(th)
		var z := radius * sin(th)
		var t := Transform3D(Basis().scaled(Vector3(scale, scale, scale)), Vector3(x, 0.0, z))
		multimesh.set_instance_transform(idx, t)
		multimesh.set_instance_color(idx, colour)
		idx += 1

	_live_count = idx
	multimesh.visible_instance_count = _live_count
	print("[DensityField] WITNESS: %d exact bands (width=%s), max_count=%d, capped=%s" % \
		[_live_count, BAND_WIDTH, max_count, agg.get("capped", false)])
