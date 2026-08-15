class_name DubSarTheme
extends RefCounted
# =============================================================================
# DubSarTheme — shared design tokens for the New DubSar Theater
# (dashboards, Document Explorer, DB Connector Wizard, HeptaScript
# notebook, PDM canvas). 2026-07-21.
#
# Palette extracted directly from the Architect's own reference
# screenshots (EnkiDDB Document Explorer, EnkiDDB Monitor, EnkiMDB
# Monitor) -- not invented. Every existing scene in this project
# currently defines its own small, slightly-different local palette
# (theater_controller.gd's GOLD/TEAL/AMBER/CRIMSON, pdm_tab.gd's
# C_BG/C_GEM/C_ORBIT/C_FUZZY, theater_3d.gd's C_GOLD/C_PANEL/C_DIM,
# wizard.tscn's inline StyleBoxFlat colors) -- this file is the new
# single source of truth going forward. Migrating each of those scenes
# onto it is tracked separately (restyle tasks), not done blindly here
# to avoid touching every screen's visual behavior in the same change
# that only adds the shared vocabulary.
#
# Usage: `const Theme_ := preload("res://scripts/dubsar_theme.gd")` then
# `Theme_.BG`, `Theme_.card_stylebox()`, `Theme_.stat_tile(...)`, etc.
# =============================================================================

# ── Ground ────────────────────────────────────────────────────────────
const BG           = Color("#0a0e1a")  # page background, near-black navy
const BG_ELEVATED  = Color("#0d1220")  # header/footer bars
const PANEL_BG     = Color("#131826")  # card/panel background
const PANEL_BG_HOVER = Color("#171d2e")
const BORDER       = Color("#212940")  # card border, hairline separators

# ── Accent (spend boldness here, nowhere else) ──────────────────────────
const ACCENT_CYAN   = Color("#22d3ee")  # primary accent -- headers, active tab, EnkiDB
const ACCENT_TEAL   = Color("#00b894")  # EnkiDDB brand accent
const ACCENT_AMBER  = Color("#fbbf24")  # GOLDEN state, warnings, EnkiDB stat highlight
const ACCENT_GREEN  = Color("#4ade80")  # success, EnkiMDB brand accent
const ACCENT_PURPLE = Color("#8b7ff5")  # secondary series (Identity-Kaki line, etc.)
const ACCENT_RED    = Color("#f87171")  # blocked / dead / error

# ── Text ──────────────────────────────────────────────────────────────
const TEXT_PRIMARY = Color("#e8eaf0")
const TEXT_DIM     = Color("#8b93a7")
const TEXT_FAINT   = Color("#5b6478")

# ── Semantic (particle/document state -- separate from the accent hue) ──
const STATE_GOLDEN = ACCENT_AMBER
const STATE_FUZZY  = Color("#f59e0b")
const STATE_DEAD   = TEXT_FAINT
const STATE_BLOCKED = ACCENT_RED

# ── Type scale ────────────────────────────────────────────────────────
# BUMPED 2026-08-02 (Architect's own report, live, on real hardware --
# these were genuinely too small to read comfortably, not a rendering
# artifact of any screenshot). Roughly 1.4-1.5x the previous values.
const SIZE_STAT_NUMBER = 36
const SIZE_TITLE       = 24
const SIZE_BODY        = 17
const SIZE_LABEL       = 15   # uppercase small labels (tile captions, column headers)
const SIZE_CODE        = 17

# ── Per-engine brand color (matches enki_engines.gd's TABLE, kept in
# sync manually -- that file is the authority on which engine has which
# color; this is only a fallback for callers that don't want to
# preload EnkiEngines just to read one field). ──
const ENGINE_COLOR = {
	"EnkiDB":  ACCENT_CYAN,
	"EnkiDW":  Color("#45b7d1"),
	"EnkiSDB": Color("#f9a826"),
	"EnkiODB": ACCENT_PURPLE,
	"EnkiQDB": Color("#fd79a8"),
	"EnkiMDB": Color("#e17055"),
	"EnkiDDB": ACCENT_TEAL,
}

# ═════════════════════════════════════════════════════════════════════
# Stylebox builders
# ═════════════════════════════════════════════════════════════════════

static func card_stylebox(radius: int = 12, border_color: Color = BORDER) -> StyleBoxFlat:
	var sb = StyleBoxFlat.new()
	sb.bg_color = PANEL_BG
	sb.set_corner_radius_all(radius)
	sb.set_border_width_all(1)
	sb.border_color = border_color
	sb.content_margin_left = 16
	sb.content_margin_top = 14
	sb.content_margin_right = 16
	sb.content_margin_bottom = 14
	return sb

static func header_stylebox() -> StyleBoxFlat:
	var sb = StyleBoxFlat.new()
	sb.bg_color = BG_ELEVATED
	sb.border_width_bottom = 1
	sb.border_color = BORDER
	sb.content_margin_left = 20
	sb.content_margin_top = 10
	sb.content_margin_right = 20
	sb.content_margin_bottom = 10
	return sb

static func tab_stylebox(active: bool, accent: Color = ACCENT_CYAN) -> StyleBoxFlat:
	var sb = StyleBoxFlat.new()
	sb.bg_color = PANEL_BG if active else Color(0, 0, 0, 0)
	sb.set_corner_radius_all(8)
	if active:
		sb.border_width_bottom = 2
		sb.border_color = accent
	sb.content_margin_left = 14
	sb.content_margin_top = 8
	sb.content_margin_right = 14
	sb.content_margin_bottom = 8
	return sb

# ═════════════════════════════════════════════════════════════════════
# Composite widgets — the pieces a base Theme resource can't cover
# (stat tiles, badges) since they're not standard Control types.
# ═════════════════════════════════════════════════════════════════════

# A dashboard stat tile: uppercase label, big number, small trend line.
# `trend_text` may be "" to omit the trend row entirely.
static func stat_tile(label: String, value_text: String, trend_text: String = "", accent: Color = ACCENT_CYAN) -> PanelContainer:
	var panel = PanelContainer.new()
	panel.add_theme_stylebox_override("panel", card_stylebox())

	var vbox = VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)
	panel.add_child(vbox)

	var lbl = Label.new()
	lbl.text = label.to_upper()
	lbl.add_theme_font_size_override("font_size", SIZE_LABEL)
	lbl.add_theme_color_override("font_color", TEXT_DIM)
	vbox.add_child(lbl)

	var val = Label.new()
	val.text = value_text
	val.add_theme_font_size_override("font_size", SIZE_STAT_NUMBER)
	val.add_theme_color_override("font_color", TEXT_PRIMARY)
	vbox.add_child(val)

	if not trend_text.is_empty():
		var trend = Label.new()
		trend.text = trend_text
		trend.add_theme_font_size_override("font_size", SIZE_LABEL)
		trend.add_theme_color_override("font_color", accent)
		vbox.add_child(trend)

	return panel

# A small colored status pill (e.g. "GOLDEN", "read-only", "blocked").
static func pill(text: String, accent: Color) -> PanelContainer:
	var panel = PanelContainer.new()
	var sb = StyleBoxFlat.new()
	sb.bg_color = Color(accent.r, accent.g, accent.b, 0.16)
	sb.set_corner_radius_all(10)
	sb.content_margin_left = 10
	sb.content_margin_top = 3
	sb.content_margin_right = 10
	sb.content_margin_bottom = 3
	panel.add_theme_stylebox_override("panel", sb)

	var lbl = Label.new()
	lbl.text = text
	lbl.add_theme_font_size_override("font_size", SIZE_LABEL)
	lbl.add_theme_color_override("font_color", accent)
	panel.add_child(lbl)
	return panel

# A left-accent-bar list row (used by "Active HeptaScript Queries",
# "Adad Gate Import Pipeline" style panels in the reference dashboards).
static func accent_row(title: String, subtitle: String, right_text: String, accent: Color) -> PanelContainer:
	var panel = PanelContainer.new()
	var sb = StyleBoxFlat.new()
	sb.bg_color = PANEL_BG
	sb.set_corner_radius_all(8)
	sb.border_width_left = 3
	sb.border_color = accent
	sb.content_margin_left = 12
	sb.content_margin_top = 8
	sb.content_margin_right = 12
	sb.content_margin_bottom = 8
	panel.add_theme_stylebox_override("panel", sb)

	var hbox = HBoxContainer.new()
	panel.add_child(hbox)

	var vbox = VBoxContainer.new()
	vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_theme_constant_override("separation", 2)
	hbox.add_child(vbox)

	var title_lbl = Label.new()
	title_lbl.text = title
	title_lbl.add_theme_font_size_override("font_size", SIZE_CODE)
	title_lbl.add_theme_color_override("font_color", TEXT_PRIMARY)
	vbox.add_child(title_lbl)

	if not subtitle.is_empty():
		var sub_lbl = Label.new()
		sub_lbl.text = subtitle
		sub_lbl.add_theme_font_size_override("font_size", SIZE_LABEL)
		sub_lbl.add_theme_color_override("font_color", TEXT_DIM)
		vbox.add_child(sub_lbl)

	if not right_text.is_empty():
		var right_lbl = Label.new()
		right_lbl.text = right_text
		right_lbl.add_theme_font_size_override("font_size", SIZE_BODY)
		right_lbl.add_theme_color_override("font_color", accent)
		right_lbl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		hbox.add_child(right_lbl)

	return panel

# Applies the base ground/text colors to any Control -- the minimum
# every new dashboard page should call once at the root.
static func apply_page_background(control: Control) -> void:
	var sb = StyleBoxFlat.new()
	sb.bg_color = BG
	control.add_theme_stylebox_override("panel", sb)
