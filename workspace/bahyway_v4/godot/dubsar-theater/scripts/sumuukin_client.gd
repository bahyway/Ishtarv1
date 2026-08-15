class_name SumuUkinClient
extends RefCounted
# =============================================================================
# ŠUMU-UKIN client-side fan-out — Godot counterpart of the real
# crates/heptascript/src/sumuukin.rs route() dispatch (2026-07-10).
#
# REWRITTEN 2026-07-21: the name is kept (per the Architect's explicit
# instruction -- "delete it but keep the name for the binary alternative
# of this protocol") but the wire protocol underneath is not the same
# one anymore. The old implementation spoke [u32 LE query_len][UTF-8
# query] request / [u32 LE frame_len]-prefixed JSON batch frames
# response, matching `enkidb-query-server` -- a crate that was never
# actually deployed anywhere in this fleet's real infrastructure. Every
# one of the 7 EnkiDB Types' real, currently-deployed Read Nodes
# (enkidb-read-server, enkiddb-read-server, enkimdb-read-server,
# enkisdb-read-server, enkiodb-read-server, enkiqdb-read-server,
# enkidw-read-server) speaks a real, hand-rolled BINARY wire protocol
# instead -- no JSON, ever, per the Architect's standing law for EnkiDB
# Types communications (see any of those crates' own `main.rs` module
# doc comment for the authoritative frame layout; they're all
# byte-identical). This file's binary decode logic is ported from
# `theater_3d.gd`'s own `EnkiDbTCP_decode_binary` (built and proven
# first against the EnkiDB target, PB-22x) rather than written twice --
# `theater_3d.gd` now defers to this file instead of keeping its own
# copy (see that file's own updated header).
#
# Request framing: `[u32 LE len]["QUERY:" + heptascript source, UTF-8]`,
# no trailing sentinel -- matches every Read Node's own `read_frame()`.
# Response framing: `[u32 LE len][payload][u32 LE 0 trailer]` -- exactly
# ONE request, ONE response frame, then the server closes the
# connection (no streaming, no DONE-sentinel loop like the old
# protocol needed). The payload's own length prefix is authoritative,
# so the client reads exactly `len` payload bytes and disconnects --
# the trailing 4 zero bytes are real (bin/enkiddb-read-server/src/
# main.rs's send_frame() genuinely writes them) but are simply never
# read, not "read and discarded": harmless on a one-response-per-
# connection protocol, since the socket closes right after.
#
# Payload tag byte: 0x00 success (row stream), 0x01 error, 0x02 search
# success (EnkiDDB's RAG only). `route()`/`_dispatch_one_sync` default to
# prefixing bare HeptaScript source with QUERY:, but pass an
# already-prefixed command straight through -- so `route(targets,
# "SEARCH:5:some phrase", policy)` reaches EnkiDDB's real SEARCH command
# the same way `graph_explorer.gd` already pre-prefixes its own QUERY:
# calls. Search hits and query rows both come back in the same "rows"
# field per target -- see decode_binary_response()'s own doc comment.
#
# TRIBES:<heptascript> (added 2026-07-21, the Architect's "PU" idea):
# every real Read Node also loads a second, small corpus -- one real
# "tribe_summary" entity per distinct tribe, attrs `tribe.id` (Int) and
# `tribe.particle_count` (Int) -- computed once at materialize time, not
# a live aggregate (the anti-SQL law is unaffected). Same 0x00 row-stream
# response shape as QUERY:, just answered from that separate corpus
# instead of the real particle one. See `crates/enkidb-readnode/src/
# materialize.rs`'s own doc comment for why it's kept in a separate Data
# File pair rather than mixed into the real particle corpus (a real
# regression this session caught and fixed: a plain WHO T.E full scan
# was silently returning tribe_summary rows mixed into real results).
#
# Targets are the 7 Enki engines (see enki_engines.gd for the real,
# currently-deployed port numbering). Dispatch to a target with no live
# server will honestly report a connection failure, never a fabricated
# success.
# =============================================================================

enum FanOutPolicy { ALL_PARALLEL, FIRST_ONLY, ALL_SERIAL }

const CONNECT_TIMEOUT_MS = 5000
const IO_TIMEOUT_MS = 30000

# targets: Array[Dictionary] each {"engine": String, "host": String, "port": int}
# Returns: {
#   "targets_contacted": int, "targets_failed": int,
#   "errors": Array[String],
#   "per_target": Array[Dictionary]  # {"engine","host","port","ok","rows","error","elapsed_ms"}
# }
static func route(targets: Array, query_text: String, policy: int) -> Dictionary:
	match policy:
		FanOutPolicy.FIRST_ONLY:
			if targets.is_empty():
				return _empty_result()
			var r = _dispatch_one_sync(targets[0], query_text)
			return _summarize([r])
		FanOutPolicy.ALL_SERIAL:
			var results: Array = []
			for t in targets:
				results.append(_dispatch_one_sync(t, query_text))
			return _summarize(results)
		FanOutPolicy.ALL_PARALLEL:
			return _dispatch_all_parallel(targets, query_text)
		_:
			return _empty_result()

static func _empty_result() -> Dictionary:
	return {"targets_contacted": 0, "targets_failed": 0, "errors": [], "per_target": []}

static func _summarize(results: Array) -> Dictionary:
	var contacted = 0
	var failed = 0
	var errors: Array = []
	for r in results:
		if r.ok:
			contacted += 1
		else:
			failed += 1
			errors.append("%s (%s:%s): %s" % [r.engine, r.host, r.port, r.error])
	return {"targets_contacted": contacted, "targets_failed": failed,
			"errors": errors, "per_target": results}

# ===== PARALLEL DISPATCH (real Thread objects, genuinely concurrent) =====

static func _dispatch_all_parallel(targets: Array, query_text: String) -> Dictionary:
	var threads: Array[Thread] = []
	var slots: Array = []
	slots.resize(targets.size())

	for i in range(targets.size()):
		slots[i] = null
		var t = Thread.new()
		threads.append(t)
		# Bind index + target so the thread writes into its own slot only.
		t.start(_thread_entry.bind(targets[i], query_text, slots, i))

	for t in threads:
		t.wait_to_finish()

	var results: Array = []
	for s in slots:
		results.append(s)
	return _summarize(results)

# Runs on a worker thread. `slots` is shared by reference (Godot Arrays are
# passed by reference), so each thread writing only its own index `i` is
# race-free without needing a Mutex.
static func _thread_entry(target: Dictionary, query_text: String, slots: Array, i: int) -> void:
	slots[i] = _dispatch_one_sync(target, query_text)

# ===== SYNCHRONOUS single-target dispatch (safe to call from a Thread) =====
# Blocking by design: worker threads cannot `await get_tree().process_frame`
# (that signal only fires on the main thread), so this polls with
# OS.delay_msec() instead.
static func _dispatch_one_sync(target: Dictionary, query_text: String) -> Dictionary:
	var start_t = Time.get_ticks_msec()
	var engine = target.get("engine", "?")
	var host = target.get("host", "127.0.0.1")
	var port = int(target.get("port", 0))

	var peer = StreamPeerTCP.new()
	var err = peer.connect_to_host(host, port)
	if err != OK:
		return _fail(engine, host, port, "connect_to_host error %d" % err, start_t)

	var t0 = Time.get_ticks_msec()
	while peer.get_status() == StreamPeerTCP.STATUS_CONNECTING:
		peer.poll()
		if Time.get_ticks_msec() - t0 > CONNECT_TIMEOUT_MS:
			peer.disconnect_from_host()
			return _fail(engine, host, port, "connect timeout", start_t)
		OS.delay_msec(5)

	if peer.get_status() != StreamPeerTCP.STATUS_CONNECTED:
		peer.disconnect_from_host()
		return _fail(engine, host, port, "connection refused or unreachable", start_t)

	# Real Read Nodes require a command prefix -- the old
	# enkidb-query-server took raw W5H2/HeptaScript text with no prefix,
	# which is why the pre-2026-07-21 version of this function sent
	# `query_text` unprefixed. See any *-read-server's own module doc for
	# the real command set (QUERY:<heptascript>, SEARCH:<top_k>:<text> on
	# EnkiDDB only, TRIBES:<heptascript> added 2026-07-21 -- the real
	# per-tribe particle-count corpus every *-read-server now also loads,
	# see enkidb_readnode::open_tribe_summaries). Callers may already pass
	# an explicitly-prefixed command (heptascript_editor.gd's own
	# _format_row() anticipates SEARCH: hits arriving through this same
	# path) -- only default to QUERY: when the caller passed bare
	# HeptaScript source.
	var command = query_text
	if not (command.begins_with("QUERY:") or command.begins_with("SEARCH:") or command.begins_with("TRIBES:")):
		command = "QUERY:" + command
	var body = command.to_utf8_buffer()
	var len_bytes = PackedByteArray()
	len_bytes.resize(4)
	var qlen = body.size()
	len_bytes[0] = qlen & 0xFF
	len_bytes[1] = (qlen >> 8) & 0xFF
	len_bytes[2] = (qlen >> 16) & 0xFF
	len_bytes[3] = (qlen >> 24) & 0xFF
	peer.put_data(len_bytes)
	peer.put_data(body)

	# Read exactly ONE length-prefixed response frame -- real Read Nodes
	# answer one request with one frame and close, no DONE-sentinel loop.
	var read_start = Time.get_ticks_msec()
	while peer.get_available_bytes() < 4:
		peer.poll()
		if Time.get_ticks_msec() - read_start > IO_TIMEOUT_MS:
			peer.disconnect_from_host()
			return _fail(engine, host, port, "timed out waiting for response header", start_t)
		OS.delay_msec(5)

	var lb_result = peer.get_data(4)
	if lb_result[0] != OK:
		peer.disconnect_from_host()
		return _fail(engine, host, port, "frame length read error", start_t)
	var lb_bytes: PackedByteArray = lb_result[1]
	var frame_len: int = lb_bytes[0] | (lb_bytes[1] << 8) | (lb_bytes[2] << 16) | (lb_bytes[3] << 24)

	while peer.get_available_bytes() < frame_len:
		peer.poll()
		OS.delay_msec(5)
		if Time.get_ticks_msec() - read_start > IO_TIMEOUT_MS:
			peer.disconnect_from_host()
			return _fail(engine, host, port, "frame body read timeout", start_t)

	var fr = peer.get_data(frame_len)
	peer.disconnect_from_host()
	if fr[0] != OK:
		return _fail(engine, host, port, "frame body read error", start_t)

	var raw: PackedByteArray = fr[1]
	var decoded = decode_binary_response(raw)
	if not decoded.error.is_empty():
		return _fail(engine, host, port, decoded.error, start_t)

	# Unified into one "rows" field regardless of which real response tag
	# came back -- QUERY rows carry {"kaki","attrs",...}, SEARCH hits
	# carry {"kaki","score","text"} (tag 0x02, EnkiDDB's RAG only).
	# heptascript_editor.gd's own _format_row() already distinguishes the
	# two shapes by which key is present, not by which command was sent.
	var rows: Array = decoded.rows if decoded.rows.size() > 0 else decoded.hits

	return {
		"engine": engine, "host": host, "port": port,
		"ok": true, "rows": rows, "error": "",
		"elapsed_ms": Time.get_ticks_msec() - start_t,
	}

static func _fail(engine: String, host: String, port: int, msg: String, start_t: int) -> Dictionary:
	return {
		"engine": engine, "host": host, "port": port,
		"ok": false, "rows": [], "error": msg,
		"elapsed_ms": Time.get_ticks_msec() - start_t,
	}

# ===== Binary wire decode (real frame layout, every *-read-server alike) =====
#
# Ported from theater_3d.gd's own EnkiDbTCP_decode_binary (built and
# live-verified first, PB-22x) so there is exactly one copy of this
# decoder instead of a second one drifting out of sync -- theater_3d.gd
# now calls this function too.
#
#   [u8 tag] 0x00 success -> [u32 row_count] {rows} [u8 trailer_tag] {trailer}
#                            row = [16B kaki][u16 attr_count]{attrs}[u16 hist_count]{hist}
#                            attr = [u8 name_len][name][u8 type_tag][value]
#                              0=Null 1=Bool(1B) 2=Int(8B s64) 3=Float(8B double)
#                              4=Text(u32 len+bytes) 5=KakiPk(16B, hex-rendered)
#                            hist entry = [u32 epoch][u16 attr_count]{attrs}
#                            trailer_tag: 0=none 1=sync_fingerprint 2=witness_digest
#                              (non-zero followed by [u8 len][bytes], hex string)
#                    0x01 error   -> [u32 len][utf8 bytes]
#                    0x02 search  -> [u32 hit_count] {hits}  (EnkiDDB RAG only)
#                            hit = [16B kaki][4B f32 score][u32 text_len][utf8 text]
#
# Row shape returned: {"kaki": hex_string, "attrs": [[name, value], ...],
# "history": [{"epoch": int, "attrs": [...]}]} -- every downstream
# consumer (heptascript_editor.gd's log, pdm_tab.gd's grid,
# graph_explorer.gd, theater_3d.gd's orbit/grid population) already
# expects exactly this shape.
static func decode_binary_response(raw: PackedByteArray) -> Dictionary:
	if raw.size() < 1:
		return {"rows": [], "hits": [], "trailer": null, "error": "empty binary response"}
	var pos := 0
	var tag := raw.decode_u8(pos); pos += 1

	if tag == 0x01:
		if raw.size() < pos + 4:
			return {"rows": [], "hits": [], "trailer": null, "error": "truncated error frame"}
		var mlen := raw.decode_u32(pos); pos += 4
		var msg := raw.slice(pos, pos + mlen).get_string_from_utf8()
		return {"rows": [], "hits": [], "trailer": null, "error": msg}

	if tag == 0x02:
		var hit_count := raw.decode_u32(pos); pos += 4
		var hits: Array = []
		for _i in range(hit_count):
			var kaki_hex := _hex(raw.slice(pos, pos + 16)); pos += 16
			var score := raw.decode_float(pos); pos += 4
			var tlen := raw.decode_u32(pos); pos += 4
			var text := raw.slice(pos, pos + tlen).get_string_from_utf8(); pos += tlen
			hits.append({"kaki": kaki_hex, "score": score, "text": text})
		return {"rows": [], "hits": hits, "trailer": null, "error": ""}

	if tag != 0x00:
		return {"rows": [], "hits": [], "trailer": null, "error": "unrecognized binary response tag %d" % tag}

	var row_count := raw.decode_u32(pos); pos += 4
	var rows: Array = []
	for _i in range(row_count):
		var kaki_hex := _hex(raw.slice(pos, pos + 16)); pos += 16

		var attrs: Array = []
		pos = _decode_attrs(raw, pos, attrs)

		var hist_count := raw.decode_u16(pos); pos += 2
		var history: Array = []
		for _h in range(hist_count):
			var epoch := raw.decode_u32(pos); pos += 4
			var hattrs: Array = []
			pos = _decode_attrs(raw, pos, hattrs)
			history.append({"epoch": epoch, "attrs": hattrs})

		rows.append({"kaki": kaki_hex, "attrs": attrs, "history": history})

	var trailer = null
	if pos < raw.size():
		var trailer_tag := raw.decode_u8(pos); pos += 1
		if trailer_tag == 1 or trailer_tag == 2:
			var tlen := raw.decode_u8(pos); pos += 1
			var tval := raw.slice(pos, pos + tlen).get_string_from_utf8(); pos += tlen
			trailer = {"kind": "sync_fingerprint" if trailer_tag == 1 else "witness_digest", "value": tval}

	# MEASURE/GRAVITY aggregate trailer (added 2026-07-29, the "vaex
	# pattern" from the ParaView/vaex/HOOMD-blue patterns validation):
	# every real Read Node's `encode_aggregate()` (see any read-server's
	# `main.rs`, e.g. `bin/enkiddb-read-server/src/main.rs`) has appended
	# this after the verb trailer since the aggregate-verb wiring
	# (PB-241-class work), but until now no Godot client ever read past
	# the verb trailer to find it -- the bytes were sent, framed
	# correctly, and simply never decoded. Layout (must match
	# `encode_aggregate`/`encode_measure_value`/`encode_gravity_key`
	# exactly): `[u8 aggregate_tag]` 0=none 1=measured 2=grouped;
	# measured: `[measure_value]`; grouped: `[u8 capped][u32 LE
	# group_count]` then per group `[gravity_key][u64 LE count]
	# [measure_value]`; measure_value = `[u8 kind][payload]` (0=Dense u64
	# LE, 1=Flux f64 LE, 2=RotorMean f64 LE); gravity_key = `[u8 key_tag]
	# [payload]` (0=Band i64 LE, 1=Exact u32 LE len + utf8, 2=Missing,
	# 3=Overflow).
	var aggregate = {"kind": "none"}
	if pos < raw.size():
		var agg_tag := raw.decode_u8(pos); pos += 1
		if agg_tag == 1:
			var mv := _decode_measure_value(raw, pos)
			aggregate = {"kind": "measured", "value": mv[0]}
			pos = mv[1]
		elif agg_tag == 2:
			var capped := raw.decode_u8(pos) != 0; pos += 1
			var group_count := raw.decode_u32(pos); pos += 4
			var groups: Array = []
			for _g in range(group_count):
				var key := _decode_gravity_key(raw, pos)
				pos = key[1]
				var count := raw.decode_u64(pos); pos += 8
				var mv := _decode_measure_value(raw, pos)
				pos = mv[1]
				groups.append({"key": key[0], "count": count, "value": mv[0]})
			aggregate = {"kind": "grouped", "capped": capped, "groups": groups}

	return {"rows": rows, "hits": [], "trailer": trailer, "aggregate": aggregate, "error": ""}

# Decodes one `measure_value` at `pos`. Returns `[value, new_pos]` (a
# 2-element Array standing in for a tuple, same "no native tuple return"
# idiom this file already uses for `_decode_attrs`'s out-parameter, just
# as a return value here instead since there's exactly one value to hand
# back). `value` is an int for Dense, a float for Flux/RotorMean.
static func _decode_measure_value(raw: PackedByteArray, pos: int) -> Array:
	var kind := raw.decode_u8(pos); pos += 1
	match kind:
		0:
			var v := raw.decode_u64(pos); pos += 8
			return [v, pos]
		1, 2:
			var v := raw.decode_double(pos); pos += 8
			return [v, pos]
		_:
			return [null, pos]

# Decodes one `gravity_key` at `pos` into `[{"tag": String, "value": ...},
# new_pos]` ("band"->int, "exact"->String, "missing"/"overflow"->null).
static func _decode_gravity_key(raw: PackedByteArray, pos: int) -> Array:
	var tag := raw.decode_u8(pos); pos += 1
	match tag:
		0:
			var v := raw.decode_s64(pos); pos += 8
			return [{"tag": "band", "value": v}, pos]
		1:
			var slen := raw.decode_u32(pos); pos += 4
			var s := raw.slice(pos, pos + slen).get_string_from_utf8(); pos += slen
			return [{"tag": "exact", "value": s}, pos]
		2:
			return [{"tag": "missing", "value": null}, pos]
		3:
			return [{"tag": "overflow", "value": null}, pos]
		_:
			return [{"tag": "unknown", "value": null}, pos]

# Decodes one attr list starting at `pos`, appending [name, value] pairs
# into `out` (an Array passed by reference, GDScript's own idiom for an
# out-parameter since it has no tuple return). Returns the new position.
static func _decode_attrs(raw: PackedByteArray, pos: int, out: Array) -> int:
	var count := raw.decode_u16(pos); pos += 2
	for _i in range(count):
		var name_len := raw.decode_u8(pos); pos += 1
		var name := raw.slice(pos, pos + name_len).get_string_from_utf8(); pos += name_len
		var type_tag := raw.decode_u8(pos); pos += 1
		var value
		match type_tag:
			0:
				value = null
			1:
				value = raw.decode_u8(pos) != 0; pos += 1
			2:
				value = raw.decode_s64(pos); pos += 8
			3:
				value = raw.decode_double(pos); pos += 8
			4:
				var slen := raw.decode_u32(pos); pos += 4
				value = raw.slice(pos, pos + slen).get_string_from_utf8(); pos += slen
			5:
				value = _hex(raw.slice(pos, pos + 16)); pos += 16
			_:
				value = "<unknown type tag %d>" % type_tag
		out.append([name, value])
	return pos

static func _hex(bytes: PackedByteArray) -> String:
	var s := ""
	for b in bytes:
		s += "%02x" % b
	return s
