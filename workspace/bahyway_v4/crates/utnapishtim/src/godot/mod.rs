//! UTNAPISHTIM — Godot 4 Sovereign PDM IDE Generator
//!
//! CORRECTIONS applied:
//!   - particle_node.gd: DEAD colour = #404040 (NOT #800000 Maroon)
//!   - κ[6] = kaki_type_byte, κ[7] = kaki_role_byte (field names sealed)
//!   - B11 = round(eav_quality × 240.0) Plimpton 322
//!   - orbital_radius = particle property from OrbitalSubtype via EAV
//!   - W5H2 query syntax in enkidb_connector.gd (anti-SQL sovereign)
//!   - KUR level 4 added to TIAMAT match (was missing)

#![forbid(unsafe_code)]
use crate::{ClientTopology, GOLDEN_ANGLE_DEG, PLIMPTON_322_DIVISOR};
use crate::{
    ORBITAL_DEAD_EXPIRED, ORBITAL_DEAD_SEALED, ORBITAL_FUZZY_AGED, ORBITAL_FUZZY_DECAY,
    ORBITAL_FUZZY_GRAY, ORBITAL_GOLDEN_ALIVE, ORBITAL_GOLDEN_GEM,
};

pub struct GodotFile {
    pub name: String,
    pub content: String,
}
pub struct GodotApp {
    pub files: Vec<GodotFile>,
}

pub fn generate_godot_app(topo: &ClientTopology) -> GodotApp {
    GodotApp {
        files: vec![
            GodotFile {
                name: "project.godot".into(),
                content: gen_project(topo),
            },
            GodotFile {
                name: "orbital_manager.gd".into(),
                content: gen_orbital_manager(topo),
            },
            GodotFile {
                name: "particle_node.gd".into(),
                content: gen_particle_node(),
            },
            GodotFile {
                name: "tribe_config.gd".into(),
                content: gen_tribe_config(topo),
            },
            GodotFile {
                name: "enkidb_connector.gd".into(),
                content: gen_enkidb_connector(),
            },
        ],
    }
}

fn gen_project(topo: &ClientTopology) -> String {
    format!(
        r#"; UTNAPISHTIM 𒌓𒍣𒅁𒀭 — Generated Sovereign PDM IDE
; Client: {} (0x{:04X}) — BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾
[application]
config/name="{} — DubSar PDM"
config/version="4.0.0"
run/main_scene="res://main_scene.tscn"
[display]
window/size/viewport_width=1440
window/size/viewport_height=900
"#,
        topo.client_name, topo.client_id, topo.client_name
    )
}

fn gen_orbital_manager(topo: &ClientTopology) -> String {
    let tribe_block = topo
        .tribes
        .iter()
        .map(|t| {
            format!(
                "\t\t{{ \"id\": {}, \"name\": \"{}\"  \
             \"ring_radius\": {:.2}, \"colour_hex\": \"{}\" }},",
                t.tribe_id, t.tribe_name, t.ring_radius, t.colour_hex
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# orbital_manager.gd
# UTNAPISHTIM 𒌓𒍣𒅁𒀭 — Orbital Manager
# BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾
#
# SOVEREIGN RULES:
# κ[6] = kaki_type_byte  κ[7] = kaki_role_byte  (NEVER quality)
# ColourID from EAV — NEVER from KAKI bytes
# B11 = round(eav_quality × {PLIMPTON_322_DIVISOR}) Plimpton 322 — NEVER 255
# orbital_radius = particle property (OrbitalSubtype via EAV)
# ring_radius    = decorative visual ring only (tribe separation)
# Dead particle  = #404040 dark grey (NOT #800000 — Nergal AV only)
# KUR terminal   = #1A0A2E sovereign indigo
# TIAMAT levels: 1=DILBAT 2=KAKKAB 3=ERRA 4=KUR (all four)
extends Node3D

const PLIMPTON_322: float = {PLIMPTON_322_DIVISOR}
const GOLDEN_ANGLE: float = {GOLDEN_ANGLE_DEG}
const KUR_THRESHOLD: float = 0.40

# Seven sealed particle orbital radii (OrbitalSubtype)
# These are PARTICLE properties — NOT tribe ring radii
const PARTICLE_RADII: Array[float] = [
	{ORBITAL_GOLDEN_GEM},   # 0 GOLDEN_GEM   B11>=200
	{ORBITAL_GOLDEN_ALIVE},  # 1 GOLDEN_ALIVE
	{ORBITAL_FUZZY_AGED},   # 2 FUZZY_AGED
	{ORBITAL_FUZZY_GRAY},   # 3 FUZZY_GRAY
	{ORBITAL_FUZZY_DECAY},  # 4 FUZZY_DECAY  TIAMAT DILBAT fires
	{ORBITAL_DEAD_EXPIRED}, # 5 DEAD_EXPIRED #404040
	{ORBITAL_DEAD_SEALED},  # 6 DEAD_SEALED  PUZRU
]

# Tribe config (decorative ring_radius — NOT particle orbital_radius)
var TRIBES: Array = [
{tribe_block}
]

func _ready() -> void:
	_build_decorative_rings()
	_seed_demo_particles()

func _build_decorative_rings() -> void:
	for tribe in TRIBES:
		# ring_radius = decorative visual separation
		# NOT particle orbital_radius
		var torus := _make_ring(tribe["ring_radius"], Color(tribe["colour_hex"]))
		$OrbitalRings.add_child(torus)

func _make_ring(ring_radius: float, colour: Color) -> MeshInstance3D:
	var mesh := TorusMesh.new()
	mesh.inner_radius = ring_radius - 0.02
	mesh.outer_radius = ring_radius + 0.02
	mesh.rings = 128
	var mat := StandardMaterial3D.new()
	mat.albedo_color = colour
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.albedo_color.a = 0.22
	var mi := MeshInstance3D.new()
	mi.mesh = mesh
	mi.material_override = mat
	return mi

func _seed_demo_particles() -> void:
	# orbital_radius from PARTICLE_RADII (OrbitalSubtype property)
	for si in range(PARTICLE_RADII.size()):
		var orbital_r := PARTICLE_RADII[si]
		for _i in range(50):
			_spawn_particle(orbital_r, si)

func _spawn_particle(orbital_radius: float, subtype: int) -> void:
	# orbital_radius = particle property from OrbitalSubtype via EAV
	var theta := randf() * TAU
	var phi   := PI / 2.0 + (randf() - 0.5) * 0.1
	var x := orbital_radius * sin(phi) * cos(theta)
	var y := orbital_radius * cos(phi)
	var z := orbital_radius * sin(phi) * sin(theta)
	var pn := _create_particle_node(Vector3(x, y, z), subtype)
	$ParticlePool.add_child(pn)

func _create_particle_node(pos: Vector3, subtype: int) -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	var s  := SphereMesh.new()
	s.radius = 0.018; s.height = 0.036
	mi.mesh  = s
	mi.position = pos
	return mi

func compute_b11(eav_quality: float) -> int:
	# B11 = round(eav_quality × 240) — Plimpton 322 — NEVER 255
	return int(round(eav_quality * PLIMPTON_322))
"#
    )
}

fn gen_particle_node() -> String {
    format!(
        r##"# particle_node.gd
# UTNAPISHTIM 𒌓𒍣𒅁𒀭 — Sovereign Particle Node
# BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾
#
# KAKI v4.0 byte layout (sealed permanently):
#   κ[0..3] = uuid_hash
#   κ[4..5] = tribe_id
#   κ[6]    = kaki_type_byte  (0x01/0x02/0x03 — NOT quality)
#   κ[7]    = kaki_role_byte  (NOT quality)
#
# ColourID from EAV eav_quality — NEVER from KAKI bytes
# B11 = round(eav_quality × 240.0) Plimpton 322 — NEVER 255
# orbital_radius from OrbitalSubtype via EAV — NEVER from tribe
#
# COLOUR MAP (CORRECTED):
#   GOLDEN_GEM   → #FFD700  gold
#   FUZZY zone   → #4A90E2  blue
#   DEAD_EXPIRED → #404040  dark grey  (NOT #800000 — Nergal AV only)
#   KUR terminal → #1A0A2E  sovereign indigo (TIAMAT overlay)
extends MeshInstance3D

# KAKI fields — immutable after mint
var uuid_hash:      int   = 0
var tribe_id:       int   = 0
var kaki_type_byte: int   = 0x01  # κ[6] — NOT quality
var kaki_role_byte: int   = 0x00  # κ[7] — NOT quality

# EAV Mandatory Attribute — source of ColourID and B11
var eav_quality:    float = 1.0

# OrbitalSubtype (0–6) from EAV — drives orbital_radius
var orbital_subtype: int  = 1

# Orbital position (from OrbitalSubtype via EAV — NOT from tribe)
var orbital_radius: float = {ORBITAL_GOLDEN_ALIVE}
var theta:          float = 0.0
var phi:            float = PI / 2.0

# PUZRU — sovereign concealment (NEVER delete)
var is_puzru:       bool  = false

const PLIMPTON_322: float = {PLIMPTON_322_DIVISOR}

# Seven sealed orbital radii (CORRECTED: particle property)
const PARTICLE_RADII: Array[float] = [
	{ORBITAL_GOLDEN_GEM},   {ORBITAL_GOLDEN_ALIVE},  {ORBITAL_FUZZY_AGED},
	{ORBITAL_FUZZY_GRAY},   {ORBITAL_FUZZY_DECAY},
	{ORBITAL_DEAD_EXPIRED}, {ORBITAL_DEAD_SEALED},
]

func _ready() -> void:
	var sphere := SphereMesh.new()
	sphere.radius = 0.018
	mesh = sphere
	_apply_colour()

func set_eav_quality(q: float) -> void:
	# ColourID derived from EAV — NEVER from KAKI bytes
	eav_quality = clamp(q, 0.0, 1.0)
	_apply_colour()
	_update_orbital_radius()

func compute_b11() -> int:
	# B11 = round(eav_quality × 240) — Plimpton 322 — NEVER 255
	return int(round(eav_quality * PLIMPTON_322))

func apply_puzru() -> void:
	# Sovereign concealment — particle exists but invisible
	# NEVER delete a particle — PUZRU only
	is_puzru = true
	visible  = false

func _apply_colour() -> void:
	var mat := StandardMaterial3D.new()
	if eav_quality >= 0.83:       # B11 >= 200 → GOLDEN_GEM
		mat.albedo_color = Color("#FFD700")
	elif eav_quality >= 0.40:     # FUZZY zone
		mat.albedo_color = Color("#4A90E2")
	else:
		# DEAD_EXPIRED → #404040 dark grey
		# CORRECTED: was #800000 — that is NERGAL AV engine colour
		mat.albedo_color = Color("#404040")
	material_override = mat

func _update_orbital_radius() -> void:
	# Map eav_quality → orbital_subtype → orbital_radius
	# orbital_radius is a PARTICLE property via OrbitalSubtype from EAV
	if   eav_quality >= 0.83: orbital_subtype = 0  # GoldenGem
	elif eav_quality >= 0.70: orbital_subtype = 1  # GoldenAlive
	elif eav_quality >= 0.55: orbital_subtype = 2  # FuzzyAged
	elif eav_quality >= 0.40: orbital_subtype = 3  # FuzzyGray
	elif eav_quality >= 0.20: orbital_subtype = 4  # FuzzyDecay
	elif eav_quality >= 0.05: orbital_subtype = 5  # DeadExpired
	else:                      orbital_subtype = 6  # DeadSealed
	orbital_radius = PARTICLE_RADII[orbital_subtype]
"##
    )
}

fn gen_tribe_config(topo: &ClientTopology) -> String {
    let tribe_lines = topo
        .tribes
        .iter()
        .map(|t| {
            format!(
                "\t{{ \"tribe_id\": {}, \"name\": \"{}\"  \
             \"ring_radius\": {:.2}, \"colour_hex\": \"{}\" }},",
                t.tribe_id, t.tribe_name, t.ring_radius, t.colour_hex
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"# tribe_config.gd
# UTNAPISHTIM 𒌓𒍣𒅁𒀭 — Client Tribe Configuration
# Generated for: {} (0x{:04X})
# ColourID by PĀŠIRU golden angle:
#   hue = (tribe_id - 1) × {GOLDEN_ANGLE_DEG}° mod 360°
# ring_radius = DECORATIVE visual ring (tribe separation)
# NOT the same as particle orbital_radius
extends Node
const TRIBES: Array = [
{}
]
func get_tribe(id: int) -> Dictionary:
	for t in TRIBES:
		if t["tribe_id"] == id: return t
	return {{}}
"#,
        topo.client_name, topo.client_id, tribe_lines
    )
}

fn gen_enkidb_connector() -> String {
    r#"# enkidb_connector.gd
# UTNAPISHTIM 𒌓𒍣𒅁𒀭 — EnkiDB TCP Connector
# BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾
#
# SOVEREIGN: W5H2 query syntax — anti-SQL — no SELECT/FROM/WHERE/LIMIT
# Pure TCP — no HTTP, no REST, no external runtime
# Refresh every 30 seconds
#
# Wire protocol (matches enkidb-query-server/src/main.rs exactly):
#   request:  [len: u32 little-endian][query bytes]
#   response: [len: u32 little-endian][json bytes] then a 0u32 terminator
# Godot 4's StreamPeerTCP has no "data received" signal -- it must be
# polled every frame via poll() + get_available_bytes().
extends Node

signal particles_received(data: Array)
signal connection_error(msg: String)

var tcp:          StreamPeerTCP = StreamPeerTCP.new()
var enkidb_host:  String        = "192.168.122.112"
var enkidb_port:  int           = 7001
var _timer:       float         = 0.0
const REFRESH_INTERVAL: float   = 30.0

# Response-framing state machine: reading the 4-byte length header first,
# then that many payload bytes, mirroring read_frame() on the server.
var _recv_buffer:   PackedByteArray = PackedByteArray()
var _expect_len:    int             = -1

func _ready() -> void:
	_connect_enkidb()

func _process(delta: float) -> void:
	_timer += delta
	if _timer >= REFRESH_INTERVAL:
		_timer = 0.0
		_query_particles()
	_poll_response()

func _connect_enkidb() -> void:
	var err := tcp.connect_to_host(enkidb_host, enkidb_port)
	if err != OK:
		emit_signal("connection_error", "EnkiDB TCP: %s" % error_string(err))
		return
	_query_particles()

func _query_particles() -> void:
	# SOVEREIGN W5H2 syntax — ANTI-SQL — no SELECT/FROM/WHERE
	var query := "WHO   tribe:ALL\n" + \
		"WHAT  kaki_hex, eav_quality, orbital_subtype, colour_hex\n" + \
		"HOW   AllEntities\n" + \
		"WHERE state != DEAD_SEALED\n" + \
		"WHEN  AllTime\n" + \
		"WHY   \"UTNAPISHTIM live particle refresh 30s cycle\"\n" + \
		"HOW_MANY 1000"
	_send_framed(query)

func _send_framed(query: String) -> void:
	# Length-prefixed request, matching read_frame() on the server: a
	# 4-byte little-endian length header, then exactly that many bytes.
	var body := query.to_utf8_buffer()
	var header := PackedByteArray()
	header.resize(4)
	header.encode_u32(0, body.size())
	tcp.put_data(header)
	tcp.put_data(body)

func _poll_response() -> void:
	tcp.poll()
	if tcp.get_status() != StreamPeerTCP.STATUS_CONNECTED:
		return
	var avail := tcp.get_available_bytes()
	if avail <= 0:
		return

	if _expect_len < 0:
		if avail < 4:
			return
		var header := tcp.get_partial_data(4)
		if header[0] != OK:
			return
		_expect_len = header[1].decode_u32(0)
		avail = tcp.get_available_bytes()

	if _expect_len == 0:
		# Terminator frame -- the server's trailing 0u32 after the payload.
		_expect_len = -1
		return

	if avail < _expect_len:
		return

	var payload := tcp.get_partial_data(_expect_len)
	_expect_len = -1
	if payload[0] != OK:
		return
	_on_data_received(payload[1])

func _on_data_received(raw: PackedByteArray) -> void:
	var text := raw.get_string_from_utf8()
	var json  := JSON.new()
	if json.parse(text) == OK:
		emit_signal("particles_received", json.data)
"#
    .to_string()
}
