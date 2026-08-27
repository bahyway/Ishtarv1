extends Node3D
## Hubble dive: one Curve3D, no cuts. Phases by t:
##   0.00-0.25 COSMOS  (federation particle field, wells)
##   0.25-0.45 CROSSFADE cosmos->city (LOD swap by opacity)
##   0.45-0.90 FLIGHT  (bird view along route spline, banking)
##   0.90-1.00 ANCHOR  (slow orbit over target pharmacy beacon)
## Advisory-only doctrine: HUD always shows report staleness (epsilon)
## and "call ahead" as the advised rite. No reservation, ever.

@export var dive_seconds: float = 14.0
var t: float = 0.0
var playing: bool = false
var path := Curve3D.new()
@onready var cam: Camera3D = $Camera3D

func _ready() -> void:
    # Waypoints will come from gula-federation-advisory (PB-321)
    # + route core (Wadi al-Salam RTK sibling). Placeholder spline:
    for p in [Vector3(0,900,900), Vector3(0,420,380), Vector3(60,140,120),
              Vector3(120,40,40), Vector3(220,18,-40), Vector3(300,14,-120),
              Vector3(360,10,-200), Vector3(380,8,-236)]:
        path.add_point(p)

func _process(delta: float) -> void:
    if not playing: return
    t = clamp(t + delta / dive_seconds, 0.0, 1.0)
    var d := t * path.get_baked_length()
    cam.global_position = path.sample_baked(d)
    var ahead := path.sample_baked(min(d + 12.0, path.get_baked_length()))
    cam.look_at(ahead, Vector3.UP)
    if t >= 1.0: playing = false

func begin_dive() -> void:
    t = 0.0; playing = true
