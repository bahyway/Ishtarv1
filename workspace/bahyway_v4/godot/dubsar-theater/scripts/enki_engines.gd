class_name EnkiEngines
# =============================================================================
# WIZ-001 — Single source of truth for the EnkiDB v4.0 family.
# Seven engines, HEPT binary TCP (magic 0x48455054).
#
# Port numbering reconciled 2026-07-10: preserves the numbering already
# live in the pre-existing enkidb_wizard.gd (EnkiDB=7001 .. EnkiQDB=7005),
# extended with the two engines that script never had (EnkiMDB, EnkiDDB).
# This is the ONLY wizard in dubsar-theater as of this consolidation —
# the standalone enkidb_wizard.gd/.tscn and dubsar_proof.gd's inline
# wizard are retired in its favour.
#
# PB-207 (2026-07-19): EnkiDDB and EnkiMDB ports corrected from their
# 2026-07-10 placeholders (7007/7006) to the real ports of the actually
# deployed Read Nodes (`enkiddb-read-server`/`enkimdb-read-server`, see
# playbooks/playbook_192_enkiddb_enkimdb_podman_deploy_executable.yml).
#
# 2026-07-21: the other five engines' ports are no longer placeholders --
# `playbook_221_enkidb_core_deploy_and_scale_sweep.yml` and
# `playbook_222_enkisdb_odb_qdb_dw_deploy.yml` deployed real
# enkidb-read-server (7001), enkidw-read-server (7002),
# enkisdb-read-server (7003), enkiodb-read-server (7004), and
# enkiqdb-read-server (7005) at exactly these port numbers -- confirmed
# directly against each server's own `bind_addr()` default, not assumed
# from this file's own history. Every entry below is a Read Node port
# (QUERY:/SEARCH: only) -- the matching Write Node for each type lives on
# a different port this table doesn't track (e.g. EnkiDB's own write
# side is enkidb-write-server:7011, EnkiSDB/EnkiODB/EnkiQDB's shared
# write side is enkisdb-write-server:7013, EnkiDW's is
# enkidw-write-server:7012) -- so every engine here is read_only from
# this table's point of view, not just EnkiMDB/EnkiDDB.
#
# FIXED (2026-07-27): wizard.gd's own "EnkiMDB is read-only at runtime
# (Nergal/ADAD law) -- use a Gilgamesh Passport" gate was reading THIS
# SAME read_only field to decide whether to refuse a Sargon Passport --
# meaning it refused Sargon against every single engine, not just
# EnkiMDB (confirmed live: read_only is true on all seven rows above,
# for the table-scope reason explained in the comment right above this
# one, unrelated to Nergal/ADAD). gilgamesh_required is the field that
# actually gates passport type -- true only where the law requires it
# (EnkiMDB); read_only keeps meaning exactly what it always has
# ("this table only tracks Read Node ports"), and dashboard_theater.gd's
# "READ NODE" pill (which legitimately wants true for all seven) is
# unaffected by this fix.
# Forbidden vocabulary anywhere in this project: SQL, Relational, 5432,
# Multi-Model (for EnkiMDB), or any foreign database's port/protocol.
# =============================================================================

const HEPT_MAGIC: PackedByteArray = [0x48, 0x45, 0x50, 0x54]  # "HEPT"

const TABLE: Dictionary = {
	"EnkiDB":  {"port": "7001", "desc": "Sovereign Particle Graph (flagship)", "icon": "🗄️", "color": Color("#4ecdc4"), "read_only": true, "gilgamesh_required": false},
	"EnkiDW":  {"port": "7002", "desc": "Data Warehouse",                   "icon": "📊", "color": Color("#45b7d1"), "read_only": true, "gilgamesh_required": false},
	"EnkiSDB": {"port": "7003", "desc": "Stage Database — landing zone ingest, validation sweep", "icon": "🔍", "color": Color("#f9a826"), "read_only": true, "gilgamesh_required": false},
	"EnkiODB": {"port": "7004", "desc": "Operational Database — active validated particles", "icon": "📦", "color": Color("#6c5ce7"), "read_only": true, "gilgamesh_required": false},
	"EnkiQDB": {"port": "7005", "desc": "Quarantine Archive — permanent append-only", "icon": "⚡", "color": Color("#fd79a8"), "read_only": true, "gilgamesh_required": false},
	"EnkiMDB": {"port": "7202", "desc": "Metadata Store (read-only)",       "icon": "🛡️", "color": Color("#e17055"), "read_only": true, "gilgamesh_required": true},
	"EnkiDDB": {"port": "7102", "desc": "Documentation Database — Tigris (read-only query/search node)", "icon": "🌐", "color": Color("#00b894"), "read_only": true, "gilgamesh_required": false},
}

const PASSPORT_TYPES: Array[String] = ["Gilgamesh Passport", "Sargon Passport"]
