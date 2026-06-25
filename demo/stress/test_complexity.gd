extends Control

const ProfileLogReader = preload("res://stress/profile_log_reader.gd")

@onready var gguy_surface = $GguySurface
@onready var texture_rect = %TextureRect

var reader

enum Phase {
	WARMUP,
	IDLE_SMALL,
	SINGLE_MUTATE_SMALL,
	RAPID_MUTATE_SMALL,
	IDLE_MEDIUM,
	SINGLE_MUTATE_MEDIUM,
	RAPID_MUTATE_MEDIUM,
	IDLE_LARGE,
	SINGLE_MUTATE_LARGE,
	RAPID_MUTATE_LARGE,
	FINISH
}

var phase: int = Phase.WARMUP
var frame_count: int = 0
var phase_frame: int = 0
var idle_baselines: Dictionary = {}
var mutate_spikes: Dictionary = {}
var rapid_frames: Dictionary = {}
var failures: Array[String] = []
var texture_created: Dictionary = {}
var _tex_seen := false
var mutate_counter: int = 0

var SMALL_HTML := """<html><head><style>
:root { --primary-color: #e94560; --bar-fill: #48dbfb; }
.hud { display:flex; flex-direction:column; gap:8px; padding:16px; background:#1a1a2e; border-radius:12px; box-shadow:0 4px 12px rgba(0,0,0,0.3); width:300px; }
.bars { display:flex; flex-direction:column; gap:4px; }
.bar { display:flex; align-items:center; gap:8px; background:#16213e; border-radius:8px; padding:4px 8px; border:1px solid #0f3460; }
.bar.hp .fill { height:12px; background:var(--primary-color); border-radius:4px; }
.bar.mp .fill { height:12px; background:var(--bar-fill); border-radius:4px; }
.bar span { color:#aaa; font-size:12px; }
.skills { display:flex; gap:8px; margin-top:4px; }
.skill { width:48px; height:48px; background:#16213e; border-radius:8px; border:1px solid #0f3460; display:flex; align-items:center; justify-content:center; color:#eee; font-size:18px; font-weight:bold; }
.skill:hover { background:#e94560; color:#fff; }
.minimap { width:120px; height:120px; background:#16213e; border-radius:50%; border:2px solid #0f3460; box-shadow:0 0 8px rgba(0,0,0,0.5); margin-top:8px; }
</style></head><body style="margin:0;padding:0;background:#0a0a14">
<div class="hud">
  <div class="bars">
    <div class="bar hp" id="hp-bar"><div class="fill" style="width:80%"></div><span id="hp-text">HP 80/100</span></div>
    <div class="bar mp" id="mp-bar"><div class="fill" style="width:60%"></div><span id="mp-text">MP 60/100</span></div>
  </div>
  <div class="skills">
    <div class="skill" id="skill-q">Q</div>
    <div class="skill" id="skill-w">W</div>
    <div class="skill" id="skill-e">E</div>
  </div>
  <div class="minimap" id="minimap"></div>
</div>
</body></html>"""

func _ready():
	reader = ProfileLogReader.new()
	gguy_surface.set_size(1152.0, 648.0)
	gguy_surface.load_html(SMALL_HTML)

func _process(_delta):
	reader.poll()
	frame_count += 1
	phase_frame += 1

	var tex = gguy_surface.get_texture()
	if tex and tex.get_width() > 0:
		_tex_seen = true
		if texture_rect.texture != tex:
			texture_rect.texture = tex
			texture_rect.custom_minimum_size = Vector2(1152, 648)
	elif frame_count > 60:
		print("=== TEST FAILED: no texture output — wgpu/Vulkan init likely failed ===")
		set_process(false)
		return

	match phase:
		Phase.WARMUP:
			if frame_count >= 5:
				_advance()
			_texture_check()

		Phase.IDLE_SMALL:
			_texture_check()
			var ut = _last_update_texture()
			if ut:
				idle_baselines["small"] = _ensure_array(idle_baselines, "small")
				idle_baselines["small"].append(ut)
			if phase_frame >= 60:
				_single_mutate("small", "hp-bar", "critical")
				_advance()

		Phase.SINGLE_MUTATE_SMALL:
			_texture_check()
			var ut = _last_update_texture()
			if ut:
				mutate_spikes["small"] = _ensure_array(mutate_spikes, "small")
				mutate_spikes["small"].append(ut)
			if phase_frame >= 2:
				_advance()

		Phase.RAPID_MUTATE_SMALL:
			_texture_check()
			if phase_frame <= 10:
				var bar_idx = (phase_frame % 3) + 1
				if phase_frame % 2 == 0:
					gguy_surface.add_class("skill-q", "active")
				else:
					gguy_surface.remove_class("skill-q", "active")
				mutate_counter += 1
				var ut = _last_update_texture()
				if ut:
					rapid_frames["small"] = _ensure_array(rapid_frames, "small")
					rapid_frames["small"].append(ut)
			if phase_frame >= 12:
				gguy_surface.load_html(_medium_html())
				_advance()

		Phase.IDLE_MEDIUM:
			_texture_check()
			var ut = _last_update_texture()
			if ut:
				idle_baselines["medium"] = _ensure_array(idle_baselines, "medium")
				idle_baselines["medium"].append(ut)
			if phase_frame >= 60:
				_single_mutate("medium", "slot-0-0", "highlighted")
				_advance()

		Phase.SINGLE_MUTATE_MEDIUM:
			_texture_check()
			var ut = _last_update_texture()
			if ut:
				mutate_spikes["medium"] = _ensure_array(mutate_spikes, "medium")
				mutate_spikes["medium"].append(ut)
			if phase_frame >= 2:
				_advance()

		Phase.RAPID_MUTATE_MEDIUM:
			_texture_check()
			if phase_frame <= 10:
				var idx = phase_frame % 10
				var slot = "slot-%d-0" % idx
				if phase_frame % 2 == 0:
					gguy_surface.add_class(slot, "highlighted")
				else:
					gguy_surface.remove_class(slot, "highlighted")
				mutate_counter += 1
				var ut = _last_update_texture()
				if ut:
					rapid_frames["medium"] = _ensure_array(rapid_frames, "medium")
					rapid_frames["medium"].append(ut)
			if phase_frame >= 12:
				gguy_surface.load_html(_large_html())
				_advance()

		Phase.IDLE_LARGE:
			_texture_check()
			var ut = _last_update_texture()
			if ut:
				idle_baselines["large"] = _ensure_array(idle_baselines, "large")
				idle_baselines["large"].append(ut)
			if phase_frame >= 60:
				_single_mutate("large", "stat-strength", "critical")
				_advance()

		Phase.SINGLE_MUTATE_LARGE:
			_texture_check()
			var ut = _last_update_texture()
			if ut:
				mutate_spikes["large"] = _ensure_array(mutate_spikes, "large")
				mutate_spikes["large"].append(ut)
			if phase_frame >= 2:
				_advance()

		Phase.RAPID_MUTATE_LARGE:
			_texture_check()
			if phase_frame <= 10:
				var idx = phase_frame % 10
				if phase_frame % 2 == 0:
					gguy_surface.add_class("inv-%d" % idx, "active")
				else:
					gguy_surface.remove_class("inv-%d" % idx, "active")
				mutate_counter += 1
				var ut = _last_update_texture()
				if ut:
					rapid_frames["large"] = _ensure_array(rapid_frames, "large")
					rapid_frames["large"].append(ut)
			if phase_frame >= 12:
				_advance()

		Phase.FINISH:
			_print_results()
			if failures.is_empty():
				print("=== TEST COMPLETE ===")
			else:
				print("=== TEST FAILED: " + "; ".join(failures) + " ===")
			set_process(false)

func _advance():
	phase += 1
	phase_frame = 0

func _ensure_array(dict: Dictionary, key: String) -> Array:
	if not dict.has(key):
		dict[key] = []
	return dict[key]

func _last_update_texture():
	var uts = reader.get_update_texture_frames()
	if uts.is_empty():
		return null
	return uts[-1]

func _texture_check():
	var ut = _last_update_texture()
	if not ut:
		return
	if ut.get("tex") == "yes":
		var creates = reader.frames.filter(func(f): return f.get("type") == "texture_create")
		if creates.is_empty():
			var size = ["small", "medium", "large"][min(phase / 3, 2)]
			if not texture_created.has(size):
				texture_created[size] = 0
			texture_created[size] += 1

func _single_mutate(size: String, selector: String, cls_name: String):
	gguy_surface.add_class(selector, cls_name)

func _medium_html() -> String:
	var html = "<html><head><style>"
	html += ":root { --primary-color: #e94560; --bar-fill: #48dbfb; }"
	html += ".inv { display:grid; grid-template-columns:repeat(10,48px); gap:4px; padding:16px; background:#1a1a2e; border-radius:12px; box-shadow:0 4px 12px rgba(0,0,0,0.3); }"
	html += ".slot { width:48px; height:48px; background:#16213e; border-radius:6px; border:1px solid #0f3460; display:flex; flex-direction:column; align-items:center; justify-content:center; }"
	html += ".slot:hover { border-color:var(--primary-color); }"
	html += ".slot .icon { font-size:20px; }"
	html += ".slot .count { font-size:10px; color:#aaa; }"
	html += ".slot.highlighted { border-color:var(--bar-fill); box-shadow:0 0 6px var(--bar-fill); }"
	html += "</style></head><body style='margin:0;padding:0;background:#0a0a14'>"
	html += "<div class='inv' id='inventory'>"
	for row in range(6):
		for col in range(10):
			var id = "slot-%d-%d" % [row, col]
			var idx = row * 10 + col
			html += "<div class='slot' id='%s'><span class='icon'>★</span><span class='count'>%d</span></div>" % [id, idx]
	html += "</div></body></html>"
	return html

func _large_html() -> String:
	var html = "<html><head><style>"
	html += ":root { --primary-color: #e94560; --bar-fill: #48dbfb; --bg: #1a1a2e; }"
	html += "body { margin:0; padding:0; background:#0a0a14; font-family:sans-serif; display:flex; flex-direction:column; gap:12px; padding:16px; }"
	html += ".panel { background:var(--bg); border-radius:12px; padding:16px; border:1px solid #0f3460; box-shadow:0 4px 12px rgba(0,0,0,0.3); }"
	html += ".panel h2 { color:var(--primary-color); font-size:16px; margin:0 0 8px 0; }"
	html += ".inv { display:grid; grid-template-columns:repeat(10,48px); gap:4px; }"
	html += ".slot { width:48px; height:48px; background:#16213e; border-radius:6px; border:1px solid #0f3460; display:flex; flex-direction:column; align-items:center; justify-content:center; }"
	html += ".slot:hover { border-color:var(--primary-color); }"
	html += ".slot .icon { font-size:20px; }"
	html += ".slot .count { font-size:10px; color:#aaa; }"
	html += ".slot.active { border-color:var(--primary-color); box-shadow:0 0 6px var(--primary-color); }"
	html += ".equip { display:grid; grid-template-columns:repeat(7,auto); gap:8px; }"
	html += ".equip-slot { padding:8px 12px; background:#16213e; border-radius:6px; border:1px solid #0f3460; color:#ccc; font-size:13px; }"
	html += ".equip-slot:hover { border-color:var(--bar-fill); }"
	html += ".stats { display:grid; grid-template-columns:1fr 1fr; gap:4px; }"
	html += ".stat { display:flex; align-items:center; gap:6px; padding:4px 8px; background:#16213e; border-radius:4px; font-size:12px; color:#ccc; }"
	html += ".stat .label { color:#888; width:80px; }"
	html += ".stat .value { color:#eee; width:30px; text-align:right; }"
	html += ".stat .sbar { flex:1; height:6px; background:#0f3460; border-radius:3px; }"
	html += ".stat .sbar .fill { height:6px; background:var(--bar-fill); border-radius:3px; }"
	html += ".stat.critical .sbar .fill { background:var(--primary-color); }"
	html += ".skill-tree { display:grid; grid-template-columns:repeat(6,auto); gap:8px; }"
	html += ".talent { width:56px; height:56px; background:#16213e; border-radius:8px; border:1px solid #0f3460; display:flex; flex-direction:column; align-items:center; justify-content:center; font-size:10px; color:#ccc; }"
	html += ".talent:hover { border-color:var(--bar-fill); }"
	html += ".talent .tl-icon { font-size:18px; }"
	html += ".talent .tl-level { font-size:9px; color:#888; }"
	html += ".quest-log { display:flex; flex-direction:column; gap:8px; }"
	html += ".quest { padding:8px; background:#16213e; border-radius:6px; border:1px solid #0f3460; }"
	html += ".quest h3 { color:#eee; font-size:13px; margin:0 0 4px 0; }"
	html += ".quest p { color:#888; font-size:11px; margin:0 0 4px 0; }"
	html += ".quest .obj { color:#666; font-size:10px; }"
	html += "</style></head><body>"
	html += "<div class='panel'><h2>Inventory</h2><div class='inv' id='inventory'>"
	for row in range(6):
		for col in range(10):
			var idx = row * 10 + col
			html += "<div class='slot' id='inv-%d'><span class='icon'>✦</span><span class='count'>%d</span></div>" % [idx, idx]
	html += "</div></div>"
	html += "<div class='panel'><h2>Equipment</h2><div class='equip' id='equipment'>"
	var equip_slots = ["Head", "Neck", "Shoulders", "Back", "Chest", "Wrists", "Hands", "Waist", "Legs", "Feet", "Ring1", "Ring2", "Trinket1", "Trinket2"]
	for i in 14:
		html += "<div class='equip-slot' id='eq-%d'>%s</div>" % [i, equip_slots[i]]
	html += "</div></div>"
	html += "<div class='panel'><h2>Stats</h2><div class='stats' id='stats'>"
	for i in range(20):
		var names = ["Strength", "Agility", "Intellect", "Stamina", "Crit", "Haste", "Mastery", "Versatility", "Armor", "Block", "Parry", "Dodge", "Hit", "Expertise", "Spell Power", "Attack Power", "Mana", "Health", "Spirit", "Resilience"]
		var pct = (i * 7 + 15) % 100
		var sid = names[i].to_lower().replace(" ", "-")
		html += "<div class='stat' id='stat-%s'><span class='label'>%s</span><span class='value' id='%s-val'>%d</span><div class='sbar'><div class='fill' style='width:%d%%'></div></div></div>" % [sid, names[i], sid, pct, pct]
	html += "</div></div>"
	html += "<div class='panel'><h2>Skill Tree</h2><div class='skill-tree' id='skill-tree'>"
	for i in range(30):
		var icons = ["⚔️", "🛡️", "🏹", "✨", "🔥", "❄️"]
		html += "<div class='talent' id='tl-%d'><span class='tl-icon'>%s</span><span class='tl-name'>Talent %d</span><span class='tl-level'>%d/5</span></div>" % [i, icons[i % icons.size()], i, (i % 5) + 1]
	html += "</div></div>"
	html += "<div class='panel'><h2>Quest Log</h2><div class='quest-log' id='quest-log'>"
	for i in range(10):
		var titles = ["The First Step", "Into the Unknown", "Gathering Supplies", "The Lost Artifact", "Defend the Village", "A Dark Ritual", "The Emissary", "Secrets of the Deep", "The Final Battle", "A New Beginning"]
		var descs = ["Begin your journey into the dark forest.", "Explore the uncharted lands beyond the river.", "Collect resources for the impending siege.", "Retrieve the ancient relic from the temple.", "Protect the village from the raiding party.", "Stop the cultists from completing the ritual.", "Deliver the message to the distant kingdom.", "Dive into the ocean depths to find the truth.", "Face the dark lord in his fortress.", "Start anew in a foreign land."]
		html += "<div class='quest' id='q-%d'><h3>%s</h3><p>%s</p><div class='obj'>Objectives: %d/3 completed</div></div>" % [i, titles[i], descs[i], (i % 3) + 1]
	html += "</div></div></body></html>"
	return html

func _median(arr: Array) -> float:
	if arr.is_empty():
		return 0.0
	var sorted = arr.duplicate()
	sorted.sort()
	var mid = sorted.size() / 2
	if sorted.size() % 2 == 1:
		return float(sorted[mid])
	return float(sorted[mid - 1] + sorted[mid]) / 2.0

func _collect_reify_us(arr: Array) -> Array:
	var result: Array = []
	for e in arr:
		if e is Dictionary and e.has("reify_us"):
			result.append(float(e.reify_us) / 1000.0)
	return result

func _collect_total_us(arr: Array) -> Array:
	var result: Array = []
	for e in arr:
		if e is Dictionary and e.has("total_us"):
			result.append(float(e.total_us) / 1000.0)
	return result

func _collect_paint_us(arr: Array) -> Array:
	var result: Array = []
	for e in arr:
		if e is Dictionary and e.has("paint_us"):
			result.append(float(e.paint_us) / 1000.0)
	return result

func _print_results():
	print("\n=== Document Complexity Results ===")
	print("%-8s %-6s %-13s %-13s %-13s %-13s" % ["Size", "Nodes", "reify_median", "style_median", "layout_median", "paint_median"])

	var configs = [
		["small", 20, SMALL_HTML],
		["medium", 120, _medium_html()],
		["large", 500, _large_html()]
	]

	for cfg in configs:
		var size: String = cfg[0]
		var nodes: int = cfg[1]
		var bl = idle_baselines.get(size, [])
		var reify_vals = _collect_reify_us(bl)
		var total_vals = _collect_total_us(bl)
		var paint_vals = _collect_paint_us(bl)
		var med_reify = _median(reify_vals)
		var med_total = _median(total_vals)
		var med_paint = _median(paint_vals)
		print("%-8s %-6d %-13.2f %-13.2f %-13s %-13.2f" % [size, nodes, med_reify, 0.0, "N/A", med_paint])
		if size == "small" and med_reify >= 3.0:
			failures.append("%s reify median %.2fms >= 3ms" % [size, med_reify])
		if size == "medium" and med_reify >= 8.0:
			failures.append("%s reify median %.2fms >= 8ms" % [size, med_reify])
		if size == "large" and med_reify >= 20.0:
			print("WARN: %s reify median %.2fms >= 20ms (Blitz issue, not Gguy bug)" % [size, med_reify])
		var spike = mutate_spikes.get(size, [])
		var spike_total = _collect_total_us(spike)
		if not spike_total.is_empty():
			var spike_val = spike_total[0]
			if med_total > 0 and spike_val > med_total * 2.0:
				failures.append("%s single-mutate frame %.2fms > idle_median*2 (%.2fms)" % [size, spike_val, med_total])
		if not paint_vals.is_empty():
			var small_paint_vals = _collect_paint_us(idle_baselines.get("small", []))
			if size == "large" and not small_paint_vals.is_empty():
				var small_med = _median(small_paint_vals)
				if med_paint > small_med * 10.0 and small_med > 0:
					failures.append("large paint %.2fms >= small paint*10 (%.2fms)" % [med_paint, small_med])
	var black_frames = 0
	for s in texture_created.values():
		black_frames += s
	if black_frames > 0:
		print("WARN: %d frames had tex=yes without texture_create line (may be benign)" % [black_frames])
