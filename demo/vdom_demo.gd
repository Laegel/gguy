extends Control

const El = preload("res://el.gd")

@onready var gguy_surface: Node = $GguySurface
@onready var texture_rect: TextureRect = %TextureRect
@onready var tick_timer: Timer = $TickTimer

var count: int = 0
var render_count: int = 0
var next_item_id: int = 0
var items: Array = []
var phase: String = "loading"
var loading_ticks: int = 0

var COLORS: Array = ["#e94560", "#ff6b6b", "#feca57", "#48dbfb", "#ff9ff3", "#54a0ff"]

func _ready():
	print("VDOM DEMO: _ready")
	tick_timer.timeout.connect(_on_tick)
	gguy_surface.set_size(1152.0, 648.0)
	print("VDOM DEMO: calling load_html")
	gguy_surface.load_html("<html><body></body></html>")
	print("VDOM DEMO: calling render_ui")
	render_ui()
	print("VDOM DEMO: _ready done")

var frame_count: int = 0

func _process(_delta: float) -> void:
	frame_count += 1
	var tex = gguy_surface.get_texture()
	var tw = tex.get_width()
	if tw > 0 and texture_rect.texture != tex:
		print("VDOM DEMO: frame=%d texture updated (width=%d)" % [frame_count, tw])
		texture_rect.texture = tex
		texture_rect.custom_minimum_size = Vector2(1152, 648)
	elif tw == 0 and frame_count <= 10:
		print("VDOM DEMO: frame=%d texture width=0 (still waiting)" % frame_count)

func _on_tick() -> void:
	print("VDOM DEMO: tick phase=%s loading_ticks=%d" % [phase, loading_ticks])
	if phase == "loading":
		loading_ticks += 1
		if loading_ticks >= 3:
			phase = "running"
			count = 0
		render_ui()
		return

	count += 1
	print("VDOM DEMO: tick count=%d items=%d" % [count, items.size()])

	if items.size() < 10 and count % 2 == 0:
		items.append({"id": next_item_id, "label": "Item " + str(next_item_id)})
		next_item_id += 1

	if count == 14:
		var filtered: Array = []
		for item in items:
			if item.label != "Item 3":
				filtered.append(item)
		items = filtered

	render_ui()

func render_ui() -> void:
	print("VDOM DEMO: render_ui phase=%s count=%d" % [phase, count])
	var color_index: int = int(count / 5.0) % COLORS.size()
	var main_color: String = COLORS[color_index]

	var children: Array = [
		El.h1({"style": "color:#e94560;font-size:36px;margin:0"}, [El.text("Gguy VDOM Demo")]),
	]

	if phase == "loading":
		var dots: String = ""
		for i in range(loading_ticks % 4):
			dots += "."
		children.append(El.div({
			"style": "color:#aaa;font-size:18px;padding:40px",
		}, [El.text("Loading" + dots)]))
	else:
		children.append(El.div({
			"id": "counter",
			"style": "color:%s;font-size:64px;font-weight:bold;padding:20px;background:#16213e;border-radius:12px;border:1px solid #0f3460;min-width:120px;text-align:center" % main_color,
		}, [El.text(str(count))]))

		var bar_width: int = min(count * 5, 100)
		var bar_color: String = "#48dbfb"
		if bar_width >= 50:
			bar_color = "#feca57"
		if bar_width >= 80:
			bar_color = "#e94560"

		children.append(El.div({
			"style": "width:400px;background:#16213e;border-radius:8px;padding:4px;border:1px solid #0f3460",
		}, [
			El.div({
				"id": "bar",
				"style": "height:20px;width:%d%%;background:%s;border-radius:6px" % [bar_width, bar_color],
			}),
		]))

		if items.size() > 0:
			children.append(El.div({
				"id": "items",
				"style": "display:flex;flex-direction:column;gap:4px;width:400px",
			}, _build_item_descs()))

	children.append(El.div({
		"id": "stats",
		"style": "color:rgba(255,255,255,0.4);font-size:12px;margin-top:4px",
	}, [El.text("Re-renders: 0 · Items: 0")]))

	children.append(El.div({
		"style": "color:#555;font-size:12px;margin-top:8px",
	}, [El.text("500ms auto · VDOM diff by id · Layer 1 set_text on stats")]))

	var root: Dictionary = El.div({
		"id": "app",
		"style": "display:flex;flex-direction:column;align-items:center;justify-content:center;width:100%%;height:100%%;background:#1a1a2e;font-family:sans-serif;gap:12px",
	}, children)

	gguy_surface.render(root)
	render_count += 1

	gguy_surface.set_text("stats", "Re-renders: %d · Items: %d" % [render_count, items.size()])

func _build_item_descs() -> Array:
	var descs: Array = []
	for item in items:
		var color_idx: int = item.id % COLORS.size()
		descs.append(El.div({
			"id": "item-" + str(item.id),
			"style": "padding:8px 16px;background:#16213e;border-radius:6px;border:1px solid #0f3460;color:#eee;font-size:14px;border-left:3px solid %s" % COLORS[color_idx],
		}, [El.text(item.label)]))
	return descs
