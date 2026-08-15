class_name DubSarTheme
extends RefCounted
# =============================================================================
# DubSarTheme (Cobalt2 variant) — shared design tokens for DubSar PDM.
# 2026-07-24.
#
# Same constant names and stylebox-builder API as dubsar-theater's
# dubsar_theme.gd (deliberately — every consumer script written against
# that API works unmodified against this one), but sourced from the
# Cobalt2 palette (playbooks/playbook_228_build_bahyway_codium_theme_extension.yml's
# `cobalt2` palette table) instead of dubsar_dark, per the Architect's
# request for one unified Cobalt2 identity across DubSar's separate
# standalone apps (Codium extension, GTK/desktop chrome, and now this).
#
# Usage: `const Theme_ := preload("res://scripts/dubsar_theme.gd")` then
# `Theme_.BG`, `Theme_.card_stylebox()`, `Theme_.stat_tile(...)`, etc.
# =============================================================================

# ── Ground ────────────────────────────────────────────────────────────
const BG           = Color("#193549")  # Cobalt2 bg
const BG_ELEVATED  = Color("#0d1f2d")  # Cobalt2 bg_darker -- header/footer bars
const PANEL_BG     = Color("#122738")  # Cobalt2 bg_dark -- card/panel background
const PANEL_BG_HOVER = Color("#1e4b6e")  # Cobalt2 bg_light
const BORDER       = Color("#0d3a58")  # Cobalt2 border

# ── Accent (spend boldness here, nowhere else) ──────────────────────────
const ACCENT_CYAN   = Color("#80fcff")  # Cobalt2 accent_cyan -- primary accent
const ACCENT_TEAL   = ACCENT_CYAN       # Cobalt2 has no distinct teal; reuse cyan rather than invent one
const ACCENT_AMBER  = Color("#ffc600")  # Cobalt2 accent_yellow
const ACCENT_GREEN  = Color("#3ad900")  # Cobalt2 accent_green
const ACCENT_PURPLE = Color("#967efb")  # Cobalt2 accent_purple
const ACCENT_RED    = Color("#ff628c")  # Cobalt2 accent_red
const ACCENT_ORANGE = Color("#ff9d00")  # Cobalt2 accent_orange
const ACCENT_PINK   = Color("#ff80bf")  # Cobalt2 accent_pink

# ── Text ──────────────────────────────────────────────────────────────
const TEXT_PRIMARY = Color("#ffffff")  # Cobalt2 fg
const TEXT_DIM     = Color("#aaaaaa")  # Cobalt2 fg_dim
const TEXT_FAINT   = Color("#5f7e97")  # Cobalt2 fg_dark

# ── Semantic (particle/document state -- separate from the accent hue) ──
const STATE_GOLDEN = ACCENT_AMBER
const STATE_FUZZY  = ACCENT_ORANGE
const STATE_DEAD   = TEXT_FAINT
const STATE_BLOCKED = ACCENT_RED

# ── Type scale ────────────────────────────────────────────────────────
const SIZE_STAT_NUMBER = 28
const SIZE_TITLE       = 18
const SIZE_BODY        = 13
const SIZE_LABEL       = 11   # uppercase small labels (tile captions, column headers)
const SIZE_CODE        = 13

# ═════════════════════════════════════════════════════════════════════
# Stylebox builders (identical logic to dubsar-theater's dubsar_theme.gd,
# just drawing from the Cobalt2 constants above)
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
# Composite widgets
# ═════════════════════════════════════════════════════════════════════

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

static func apply_page_background(control: Control) -> void:
	var sb = StyleBoxFlat.new()
	sb.bg_color = BG
	control.add_theme_stylebox_override("panel", sb)
