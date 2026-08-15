extends MarginContainer

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

@onready var summary_grid: VBoxContainer = $VBoxContainer/SummaryGrid

var LABEL_COLOR: Color = DubSarTheme.TEXT_DIM
var VALUE_COLOR: Color = DubSarTheme.TEXT_PRIMARY

func _ready():
	pass

func populate_summary(data: Dictionary):
	for child in summary_grid.get_children():
		child.queue_free()

	var fields = [
		["Engine", data.get("database", "—")],
		["Host", data.get("host", "—")],
		["HEPT Port", data.get("port", "—")],
		["Scope / Journal", data.get("scope", "") if not str(data.get("scope", "")).is_empty() else "—"],
		["Passport", data.get("passport_type", "—")],
		["Identity", data.get("identity", "—")],
		["Transport", "AkkadiCipherEngine — always on"],
		["Timeout", str(data.get("timeout", 30)) + "s"],
		["Save Profile", "Yes (AkkadianSeal)" if data.get("save_profile", false) else "No"],
		["Store Passphrase", "AkkadiSafeEngine vault" if data.get("save_credentials", false) else "No"]
	]

	for field in fields:
		var row = _create_summary_row(field[0], field[1])
		summary_grid.add_child(row)

func _create_summary_row(label_text: String, value_text: String) -> PanelContainer:
	var panel = PanelContainer.new()

	var style = StyleBoxFlat.new()
	style.bg_color = DubSarTheme.PANEL_BG
	style.corner_radius_top_left = 8
	style.corner_radius_top_right = 8
	style.corner_radius_bottom_left = 8
	style.corner_radius_bottom_right = 8
	panel.add_theme_stylebox_override("panel", style)

	var hbox = HBoxContainer.new()
	hbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	panel.add_child(hbox)

	var label = Label.new()
	label.text = label_text
	label.add_theme_color_override("font_color", LABEL_COLOR)
	label.add_theme_font_size_override("font_size", 13)
	hbox.add_child(label)

	var spacer = Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	hbox.add_child(spacer)

	var value = Label.new()
	value.text = value_text
	value.add_theme_color_override("font_color", VALUE_COLOR)
	value.add_theme_font_size_override("font_size", 13)
	hbox.add_child(value)

	return panel
