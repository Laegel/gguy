extends Control

@onready var gguy_surface: Node = $GguySurface
@onready var texture_rect: TextureRect = %TextureRect
@onready var tick_timer: Timer = $TickTimer

var COLORS: Array = ["#e94560", "#ff6b6b", "#feca57", "#48dbfb", "#ff9ff3", "#54a0ff"]
var count: int = 0
var items: Array = []
var next_item_id: int = 0
var render_count: int = 0
var phase: String = "loading"
var loading_ticks: int = 0

func _ready():
	tick_timer.timeout.connect(_on_tick)
	gguy_surface.set_size(1152.0, 648.0)
	gguy_surface.load_html("<html><body style='margin:0;padding:0'></body></html>")
	build_ui()

func build_ui():
	gguy_surface.create_element("body", "div", {"id":"app","style":"display:flex;flex-direction:column;align-items:center;justify-content:center;width:100%;height:100%;background:#1a1a2e;font-family:sans-serif;gap:12px"})
	gguy_surface.create_element("app", "h1", {"id":"title","style":"color:#e94560;font-size:36px;margin:0"})
	gguy_surface.set_text("title", "Gguy Direct Mutation Demo")
	gguy_surface.create_element("app", "div", {"id":"counter_label","style":"color:#aaa;font-size:18px;padding:40px"})
	gguy_surface.set_text("counter_label", "Loading...")
	gguy_surface.create_element("app", "div", {"id":"counter","style":"color:#e94560;font-size:64px;font-weight:bold;padding:20px;background:#16213e;border-radius:12px;border:1px solid #0f3460;min-width:120px;text-align:center"})
	gguy_surface.set_text("counter", "0")
	gguy_surface.create_element("app", "div", {"id":"bar_container","style":"width:400px;background:#16213e;border-radius:8px;padding:4px;border:1px solid #0f3460"})
	gguy_surface.create_element("bar_container", "div", {"id":"bar","style":"height:20px;width:0%;background:#48dbfb;border-radius:6px"})
	gguy_surface.create_element("app", "div", {"id":"items","style":"display:flex;flex-direction:column;gap:4px;width:400px"})
	gguy_surface.create_element("app", "div", {"id":"stats","style":"color:rgba(255,255,255,0.4);font-size:12px;margin-top:4px"})
	gguy_surface.set_text("stats", "Re-renders: 0 \u00b7 Items: 0")
	gguy_surface.create_element("app", "div", {"id":"footer","style":"color:#555;font-size:12px;margin-top:8px"})
	gguy_surface.set_text("footer", "500ms auto \u00b7 Direct mutations")

func _process(_delta):
	var tex = gguy_surface.get_texture()
	if tex.get_width() > 0 and texture_rect.texture != tex:
		texture_rect.texture = tex
		texture_rect.custom_minimum_size = Vector2(1152, 648)

func _on_tick():
	if phase == "loading":
		loading_ticks += 1
		var dots = ""
		for i in range(loading_ticks % 4):
			dots += "."
		gguy_surface.set_text("counter_label", "Loading" + dots)
		if loading_ticks >= 3:
			phase = "running"
			count = 0
			gguy_surface.remove_element("counter_label")
		return

	count += 1
	var color_index: int = int(count / 5.0) % COLORS.size()
	var main_color: String = COLORS[color_index]

	gguy_surface.set_text("counter", str(count))
	var counter_style: String = "color:%s;font-size:64px;font-weight:bold;padding:20px;background:#16213e;border-radius:12px;border:1px solid #0f3460;min-width:120px;text-align:center" % main_color
	gguy_surface.set_attribute("counter", "style", counter_style)

	var bar_width: int = min(count * 5, 100)
	var bar_color: String = "#48dbfb"
	if bar_width >= 50:
		bar_color = "#feca57"
	if bar_width >= 80:
		bar_color = "#e94560"
	var bar_style: String = "height:20px;width:%d%%;background:%s;border-radius:6px" % [bar_width, bar_color]
	gguy_surface.set_attribute("bar", "style", bar_style)

	if items.size() < 10 and count % 2 == 0:
		var item_id: int = next_item_id
		items.append({"id": item_id, "label": "Item " + str(item_id)})
		var color_idx: int = item_id % COLORS.size()
		var item_style: String = "padding:8px 16px;background:#16213e;border-radius:6px;border:1px solid #0f3460;color:#eee;font-size:14px;border-left:3px solid %s" % COLORS[color_idx]
		gguy_surface.create_element("items", "div", {"id":"item-" + str(item_id), "style":item_style})
		gguy_surface.set_text("item-" + str(item_id), "Item " + str(item_id))
		next_item_id += 1

	if count == 14:
		gguy_surface.remove_element("item-3")
		items = []

		for item in items:
			if item.id != 3:
				items.append(item)

	render_count += 1
	gguy_surface.set_text("stats", "Re-renders: %d \u00b7 Items: %d" % [render_count, items.size()])
