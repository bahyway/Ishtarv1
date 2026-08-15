# enkidb_tcp.gd — Sovereign TCP client for enkidb-query-server
# Pure Godot StreamPeerTCP — no external libs, no tokio
# Wire protocol: [u32 LE query_len][UTF-8 W5H2] → BATCH frames + DONE(0)
# DUB.SAR 𒁾  2026-06-21

extends Node


const DEFAULT_HOST = "192.168.122.107"
const DEFAULT_PORT = 7001
const CONNECT_TIMEOUT = 5.0  # seconds
const READ_TIMEOUT    = 120.0

# Stats from the most recently completed execute_query() call:
# {"matched": int, "evaluated": int, "aborted": bool}. Empty until a query
# whose server wraps rows in the {"rows":..., "stats":...} shape completes
# (see enkidb-query-server/src/main.rs's result_to_json()). Older server
# builds that still send a bare row array leave this empty -- callers that
# need ABORT_SCAN's aborted flag (e.g. an E-005-style check) should treat
# an empty dict as "unknown", not "false".
static var last_stats: Dictionary = {}

# Test TCP reachability — returns true/false
static func test_connection(host: String, port: int) -> bool:
    var peer = StreamPeerTCP.new()
    var err = peer.connect_to_host(host, port)
    if err != OK:
        return false
    var t = Time.get_ticks_msec()
    while peer.get_status() == StreamPeerTCP.STATUS_CONNECTING:
        peer.poll()
        if Time.get_ticks_msec() - t > CONNECT_TIMEOUT * 1000:
            peer.disconnect_from_host()
            return false
    var ok = peer.get_status() == StreamPeerTCP.STATUS_CONNECTED
    peer.disconnect_from_host()
    return ok

# Execute a W5H2 query. Returns Array of Dicts:
#   [{"kaki": "<hex>", "attrs": [["attr_name","value"], ...]}, ...]
# attrs is keyed by the plain WHAT-clause attribute name exactly as
# requested (e.g. "person.name_arabic"), NOT a hex hash -- confirmed
# directly against enkidb-query-server/src/main.rs's rows_to_json(),
# which serializes MatchedEntity.projected's (String, AkkValue) pairs
# verbatim. (An earlier version of this comment, and of dubsar_proof.gd's
# now-fixed ATTR_NAMES lookup, assumed hashed keys -- that was wrong.)
# On error returns empty array. progress_cb(msg) called during streaming.
static func execute_query(
    host: String,
    port: int,
    w5h2_src: String,
    progress_cb: Callable = Callable()
) -> Array:
    var peer = StreamPeerTCP.new()
    var err = peer.connect_to_host(host, port)
    if err != OK:
        push_error("EnkiDbTCP: connect error %d" % err)
        return []

    # Wait for connection
    var t = Time.get_ticks_msec()
    while peer.get_status() == StreamPeerTCP.STATUS_CONNECTING:
        peer.poll()
        if Time.get_ticks_msec() - t > CONNECT_TIMEOUT * 1000:
            push_error("EnkiDbTCP: connect timeout")
            return []

    if peer.get_status() != StreamPeerTCP.STATUS_CONNECTED:
        push_error("EnkiDbTCP: failed to connect")
        return []

    # Send query frame: [u32 LE len][UTF-8 bytes]
    var query_bytes = w5h2_src.to_utf8_buffer()
    var len_bytes   = PackedByteArray()
    len_bytes.resize(4)
    var qlen = query_bytes.size()
    len_bytes[0] = qlen & 0xFF
    len_bytes[1] = (qlen >> 8) & 0xFF
    len_bytes[2] = (qlen >> 16) & 0xFF
    len_bytes[3] = (qlen >> 24) & 0xFF
    peer.put_data(len_bytes)
    peer.put_data(query_bytes)

    # Read BATCH frames until DONE sentinel (u32 = 0)
    var results: Array = []
    var start_t = Time.get_ticks_msec()
    last_stats = {}

    while peer.get_status() == StreamPeerTCP.STATUS_CONNECTED:
        peer.poll()
        if Time.get_ticks_msec() - start_t > READ_TIMEOUT * 1000:
            push_error("EnkiDbTCP: read timeout")
            break

        # Need at least 4 bytes for length prefix
        if peer.get_available_bytes() < 4:
            OS.delay_msec(10)
            continue

        # Read 4-byte length prefix
        var lb_result = peer.get_data(4)
        if lb_result[0] != OK:
            break
        var lb: PackedByteArray = lb_result[1]
        var frame_len: int = lb[0] | (lb[1] << 8) | (lb[2] << 16) | (lb[3] << 24)

        # u32 = 0 → DONE sentinel
        if frame_len == 0:
            break

        # Wait for full frame
        var wait_t = Time.get_ticks_msec()
        while peer.get_available_bytes() < frame_len:
            peer.poll()
            OS.delay_msec(5)
            if Time.get_ticks_msec() - wait_t > READ_TIMEOUT * 1000:
                push_error("EnkiDbTCP: frame read timeout")
                peer.disconnect_from_host()
                return results

        var frame_result = peer.get_data(frame_len)
        if frame_result[0] != OK:
            break
        var payload = frame_result[1].get_string_from_utf8()

        # Server error
        if payload.begins_with("ERR:"):
            push_error("EnkiDbTCP server error: " + payload)
            break

        # Parse JSON batch
        var batch = JSON.parse_string(payload)
        if batch == null:
            push_error("EnkiDbTCP: JSON parse failed")
            continue
        # Current server shape: {"rows": [...], "stats": {...}}. Older
        # servers send a bare row array -- keep accepting that too so this
        # client works against either build.
        if batch is Dictionary and batch.has("rows"):
            results.append_array(batch["rows"])
            if batch.has("stats"):
                last_stats = batch["stats"]
            if progress_cb.is_valid():
                progress_cb.call("Received %d rows..." % results.size())
        elif batch is Array:
            results.append_array(batch)
            if progress_cb.is_valid():
                progress_cb.call("Received %d rows..." % results.size())

    peer.disconnect_from_host()
    return results
