extends Control

var _dragging = false
var _drag_offset = Vector2.ZERO

func _ready():
	mouse_filter = Control.MOUSE_FILTER_STOP

func _gui_input(event):
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT:
			if event.pressed:
				_dragging = true
				_drag_offset = event.position
				accept_event()
			else:
				_dragging = false

	if event is InputEventMouseMotion && _dragging:
		var screen = get_viewport_rect().size
		var new_pos = get_global_mouse_position() - _drag_offset
		new_pos.x = clampf(new_pos.x, 0.0, screen.x - size.x)
		new_pos.y = clampf(new_pos.y, 0.0, screen.y - size.y)
		global_position = new_pos
		accept_event()
