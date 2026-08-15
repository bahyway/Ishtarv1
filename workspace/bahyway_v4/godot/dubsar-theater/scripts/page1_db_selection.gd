extends MarginContainer

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

signal database_chosen(db_name: String)

@onready var grid: GridContainer = $VBoxContainer/GridContainer

var chosen_db: String = ""
var db_cards: Dictionary = {}

func _ready():
	_create_database_cards()

func _create_database_cards():
	# Single source of truth: EnkiEngines.TABLE (seven engines, HEPT ports).
	for db_name in EnkiEngines.TABLE.keys():
		var config = EnkiEngines.TABLE[db_name]
		var card = _create_card(db_name, config)
		grid.add_child(card)
		db_cards[db_name] = card

func _create_card(db_name: String, config: Dictionary) -> PanelContainer:
	var card = PanelContainer.new()
	card.custom_minimum_size = Vector2(180, 130)
	card.mouse_filter = Control.MOUSE_FILTER_STOP
	card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	var style_normal = StyleBoxFlat.new()
	style_normal.bg_color = DubSarTheme.PANEL_BG
	style_normal.corner_radius_top_left = 12
	style_normal.corner_radius_top_right = 12
	style_normal.corner_radius_bottom_left = 12
	style_normal.corner_radius_bottom_right = 12
	style_normal.border_width_left = 2
	style_normal.border_width_top = 2
	style_normal.border_width_right = 2
	style_normal.border_width_bottom = 2
	style_normal.border_color = DubSarTheme.BORDER
	card.add_theme_stylebox_override("panel", style_normal)
	card.set_meta("style_normal", style_normal)

	var vbox = VBoxContainer.new()
	vbox.alignment = BoxContainer.ALIGNMENT_CENTER
	vbox.add_theme_constant_override("separation", 8)
	card.add_child(vbox)

	var icon_label = Label.new()
	icon_label.text = config.icon
	icon_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	icon_label.add_theme_font_size_override("font_size", 36)
	vbox.add_child(icon_label)

	var name_label = Label.new()
	name_label.text = db_name
	name_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	name_label.add_theme_font_size_override("font_size", 15)
	name_label.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	vbox.add_child(name_label)

	var desc_label = Label.new()
	desc_label.text = config.desc
	desc_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	desc_label.add_theme_font_size_override("font_size", 11)
	desc_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vbox.add_child(desc_label)

	# HEPT port hint — makes foreign ports visually impossible to miss.
	var port_label = Label.new()
	port_label.text = "HEPT :" + config.port
	port_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	port_label.add_theme_font_size_override("font_size", 10)
	port_label.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
	vbox.add_child(port_label)

	card.gui_input.connect(_on_card_input.bind(db_name, card))
	return card

func _on_card_input(event: InputEvent, db_name: String, card: PanelContainer):
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_choose_database(db_name, card)

func _choose_database(db_name: String, card: PanelContainer):
	for name in db_cards:
		var c = db_cards[name]
		var normal_style = c.get_meta("style_normal") as StyleBoxFlat
		if normal_style:
			normal_style.border_color = DubSarTheme.BORDER
			normal_style.bg_color = DubSarTheme.PANEL_BG
		for child in c.get_child(0).get_children():
			if child.name == "CheckMark":
				child.queue_free()

	chosen_db = db_name
	var style = card.get_meta("style_normal") as StyleBoxFlat
	style.border_color = DubSarTheme.ACCENT_CYAN
	style.bg_color = DubSarTheme.ACCENT_CYAN.darkened(0.85)

	var check = Label.new()
	check.name = "CheckMark"
	check.text = "✓"
	check.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	check.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	check.add_theme_font_size_override("font_size", 14)
	check.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)

	var check_bg = PanelContainer.new()
	check_bg.custom_minimum_size = Vector2(24, 24)
	var check_style = StyleBoxFlat.new()
	check_style.bg_color = DubSarTheme.ACCENT_CYAN
	check_style.corner_radius_top_left = 12
	check_style.corner_radius_top_right = 12
	check_style.corner_radius_bottom_left = 12
	check_style.corner_radius_bottom_right = 12
	check_bg.add_theme_stylebox_override("panel", check_style)
	check_bg.add_child(check)

	var container = card.get_child(0)
	check_bg.position = Vector2(card.size.x - 32, 8)
	container.add_child(check_bg)

	database_chosen.emit(db_name)
