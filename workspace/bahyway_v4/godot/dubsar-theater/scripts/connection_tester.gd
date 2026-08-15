extends Node

# WIZ-001 — Async connection tester for the EnkiDB family.
#
# CORRECTED 2026-07-10: the original WIZ-001 draft assumed a 4-byte HEPT
# magic handshake ("send HEPT, expect HEPT back"). Checked directly against
# bin/enkidb-query-server/src/main.rs — no such handshake exists. The real
# wire protocol (confirmed working in scripts/enkidb_tcp.gd) is
# [u32 LE query_len][UTF-8 query] -> batched JSON frames -> u32=0 DONE.
# Sending raw magic bytes to the real server would be misread as a garbage
# length prefix, not cleanly rejected. So this tester now does what
# enkidb_tcp.gd's own test_connection() does: TCP reachability only —
# no protocol probe, since there is no lightweight probe frame defined
# server-side yet. That is an honest limitation, not a shortcut: a real
# protocol-level handshake needs to be added to enkidb-query-server first
# if a proof-of-Enki-ness check is wanted here.

signal test_finished(success: bool, message: String)

var tcp: StreamPeerTCP
var is_testing: bool = false

func test_connection(host: String, port: int, timeout_ms: int = 5000) -> void:
	if is_testing:
		return

	is_testing = true
	tcp = StreamPeerTCP.new()

	var err = tcp.connect_to_host(host, port)
	if err != OK:
		is_testing = false
		test_finished.emit(false, "Failed to initiate connection: %d" % err)
		return

	var start_time = Time.get_ticks_msec()
	while tcp.get_status() == StreamPeerTCP.STATUS_CONNECTING:
		if Time.get_ticks_msec() - start_time > timeout_ms:
			tcp.disconnect_from_host()
			is_testing = false
			test_finished.emit(false, "Connection timed out after %d ms" % timeout_ms)
			return
		await get_tree().process_frame
		tcp.poll()

	is_testing = false
	if tcp.get_status() == StreamPeerTCP.STATUS_CONNECTED:
		tcp.disconnect_from_host()
		test_finished.emit(true, "TCP reachable — protocol-level verification not yet available (no handshake frame defined server-side)")
	else:
		tcp.disconnect_from_host()
		test_finished.emit(false, "Connection refused or unreachable")

func cancel_test():
	if tcp:
		tcp.disconnect_from_host()
	is_testing = false
