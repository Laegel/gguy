extends Control

const ProfileLogReader = preload("res://stress/profile_log_reader.gd")

@onready var gguy_surface = $GguySurface
@onready var texture_rect = %TextureRect

var reader

enum Phase { WARMUP, PATTERN_A, PATTERN_B, PATTERN_C, PATTERN_D, FINISH }
var phase: int = Phase.WARMUP
var frame_count: int = 0
var phase_frame: int = 0
var failures: Array[String] = []
var _tex_seen := false

var pattern_data: Dictionary = {}
var pattern_order: Array[String] = ["A", "B", "C", "D"]
var pattern_idx: int = -1

var pattern_mutation_counts: Dictionary = {}
var pattern_reify_counts: Dictionary = {}
var phase_start_reify_count: int = 0

func _ready():
	reader = ProfileLogReader.new()
	gguy_surface.set_size(1152.0, 648.0)
	gguy_surface.load_html(_medium_html())

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

	var curr_reify_count = reader.get_reify_frames().size()
	if pattern_idx >= 0 and pattern_idx < pattern_order.size():
		var p = pattern_order[pattern_idx]
		if not pattern_reify_counts.has(p):
			pattern_reify_counts[p] = 0
		pattern_reify_counts[p] = curr_reify_count - phase_start_reify_count

	match phase:
		Phase.WARMUP:
			if phase_frame >= 5:
				_advance()
				_init_pattern()

		Phase.PATTERN_A:
			gguy_surface.set_text("stat-str-val", str(randi() % 100))
			_track_mutation("A")
			var ut = _last_update_texture()
			if ut:
				pattern_data["A"]["raw"].append(ut)
			if phase_frame >= 300:
				_advance()
				_init_pattern()

		Phase.PATTERN_B:
			for i in range(5):
				gguy_surface.set_text("stat-str-val", str(randi() % 200))
			for i in range(3):
				var pct = str(randi() % 100)
				gguy_surface.set_attribute("slot-0-%d" % i, "style", "width:%s%%" % pct)
			gguy_surface.add_class("slot-0-0", "highlighted")
			gguy_surface.remove_class("slot-0-0", "highlighted")
			_track_mutation("B")
			var ut = _last_update_texture()
			if ut:
				pattern_data["B"]["raw"].append(ut)
			if phase_frame >= 300:
				_advance()
				_init_pattern()

		Phase.PATTERN_C:
			gguy_surface.remove_class("skill-q", "active")
			gguy_surface.add_class("skill-q", "active")
			_track_mutation("C")
			var ut = _last_update_texture()
			if ut:
				pattern_data["C"]["raw"].append(ut)
			if phase_frame >= 300:
				_advance()
				_init_pattern()

		Phase.PATTERN_D:
			if phase_frame % 30 == 0:
				gguy_surface.load_html(_medium_html_rotated())
				_track_mutation("D")
			var ut = _last_update_texture()
			if ut:
				pattern_data["D"]["raw"].append(ut)
			if phase_frame >= 300:
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

func _init_pattern():
	pattern_idx += 1
	if pattern_idx >= pattern_order.size():
		return
	var p = pattern_order[pattern_idx]
	pattern_data[p] = { "raw": [], "tex_created": 0 }
	pattern_mutation_counts[p] = 0
	phase_start_reify_count = reader.get_reify_frames().size()
	pattern_reify_counts[p] = 0

func _track_mutation(p: String):
	if not pattern_mutation_counts.has(p):
		pattern_mutation_counts[p] = 0
	pattern_mutation_counts[p] += 1

func _last_update_texture():
	var uts = reader.get_update_texture_frames()
	if uts.is_empty():
		return null
	return uts[-1]

func _medium_html() -> String:
	var html = "<html><head><style>"
	html += ":root{--primary-color:#e94560;--bar-fill:#48dbfb}"
	html += ".inv{display:grid;grid-template-columns:repeat(10,48px);gap:4px;padding:16px;background:#1a1a2e;border-radius:12px;box-shadow:0 4px 12px rgba(0,0,0,0.3)}"
	html += ".slot{width:48px;height:48px;background:#16213e;border-radius:6px;border:1px solid #0f3460;display:flex;flex-direction:column;align-items:center;justify-content:center}"
	html += ".slot:hover{border-color:var(--primary-color)}"
	html += ".slot .icon{font-size:20px}.slot .count{font-size:10px;color:#aaa}"
	html += ".slot.highlighted{border-color:var(--bar-fill);box-shadow:0 0 6px var(--bar-fill)}"
	html += ".skill{width:48px;height:48px;background:#16213e;border-radius:8px;border:1px solid #0f3460;display:flex;align-items:center;justify-content:center;color:#eee;font-size:18px;font-weight:bold}"
	html += ".skill.active{border-color:var(--primary-color);box-shadow:0 0 8px var(--primary-color)}"
	html += "#stats{display:flex;flex-direction:column;gap:4px;padding:16px}"
	html += ".stat-row{display:flex;gap:8px;align-items:center;color:#ccc;font-size:13px}"
	html += "</style></head><body style='margin:0;padding:0;background:#0a0a14'>"
	html += "<div class='inv' id='inventory'>"
	for row in range(6):
		for col in range(10):
			html += "<div class='slot' id='slot-%d-%d'><span class='icon'>★</span><span class='count'>%d</span></div>" % [row, col, row * 10 + col]
	html += "</div>"
	html += "<div id='stats'><div class='stat-row'>Strength: <span id='stat-str-val'>50</span></div>"
	html += "<div class='stat-row'>Agility: <span id='stat-agi-val'>30</span></div>"
	html += "<div class='stat-row'>Intellect: <span id='stat-int-val'>40</span></div>"
	html += "<div class='stat-row'>Stamina: <span id='stat-sta-val'>80</span></div>"
	html += "<div class='stat-row'>Crit: <span id='stat-crit-val'>15</span></div></div>"
	html += "<div id='skills'><div class='skill' id='skill-q'>Q</div></div>"
	html += "</body></html>"
	return html

var _rotation: int = 0

func _medium_html_rotated() -> String:
	_rotation += 1
	var icons = ["★","♦","●","▲","■","♣"]
	var html = "<html><head><style>"
	html += ":root{--primary-color:#e94560;--bar-fill:#48dbfb}"
	html += ".inv{display:grid;grid-template-columns:repeat(10,48px);gap:4px;padding:16px;background:#1a1a2e;border-radius:12px;box-shadow:0 4px 12px rgba(0,0,0,0.3)}"
	html += ".slot{width:48px;height:48px;background:#16213e;border-radius:6px;border:1px solid #0f3460;display:flex;flex-direction:column;align-items:center;justify-content:center}"
	html += ".slot:hover{border-color:var(--primary-color)}"
	html += ".slot .icon{font-size:20px}.slot .count{font-size:10px;color:#aaa}"
	html += ".slot.highlighted{border-color:var(--bar-fill);box-shadow:0 0 6px var(--bar-fill)}"
	html += "</style></head><body style='margin:0;padding:0;background:#0a0a14'>"
	html += "<div class='inv' id='inventory'>"
	for row in range(6):
		for col in range(10):
			var idx = row * 10 + col
			html += "<div class='slot' id='slot-%d-%d'><span class='icon'>%s</span><span class='count'>%d</span></div>" % [row, col, icons[(idx + _rotation) % icons.size()], idx]
	html += "</div></body></html>"
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

func _percentile(arr: Array, pct: float) -> float:
	if arr.is_empty():
		return 0.0
	var sorted = arr.duplicate()
	sorted.sort()
	var idx = int(ceil(sorted.size() * pct / 100.0)) - 1
	idx = clampi(idx, 0, sorted.size() - 1)
	return float(sorted[idx])

func _print_results():
	print("\n=== Rapid Mutation Results ===")
	print("%-10s %-6s %-13s %-13s %-13s %-13s" % ["Pattern", "Frames", "reify_median", "p95_update", "max_update", "dirty_gate_ok"])

	for p in pattern_order:
		var data = pattern_data.get(p, {})
		var raw = data.get("raw", [])
		var total_vals: Array = []
		var reify_vals: Array = []
		for e in raw:
			if e.has("total_us"):
				total_vals.append(float(e.total_us) / 1000.0)
			if e.has("reify_us"):
				reify_vals.append(float(e.reify_us) / 1000.0)

		var med_reify = _median(reify_vals)
		var p95_total = _percentile(total_vals, 95.0)
		var max_total = total_vals.max() if not total_vals.is_empty() else 0.0

		var dirty_gate_ok := true
		if p == "A":
			var mut_count = pattern_mutation_counts.get("A", 0)
			var reify_count = pattern_reify_counts.get("A", 0)
			var diff = abs(reify_count - mut_count)
			if diff > 2:
				dirty_gate_ok = false
				failures.append("Pattern A dirty gate: reify %d vs mutations %d (diff %d > 2)" % [reify_count, mut_count, diff])

		if p != "D":
			if max_total > 25.0:
				failures.append("Pattern %s max update_texture %.2fms > 25ms" % [p, max_total])
			if p95_total > 16.0:
				failures.append("Pattern %s p95 update_texture %.2fms > 16ms" % [p, p95_total])
			var no_reify_but_tex := 0
			for e in raw:
				var entry_had_reify = e.has("reify_us") and e.get("reify_us", -1) >= 0 and e.get("reify_us", -1) > 0
				if e.get("tex") == "yes" and not entry_had_reify:
					no_reify_but_tex += 1
			if no_reify_but_tex > 0:
				failures.append("Pattern %s: %d frames with tex=yes but no reify" % [p, no_reify_but_tex])
		else:
			for i in range(1, raw.size()):
				var ut = raw[i]
				var prev_ut = raw[i - 1]
				if prev_ut.get("total_us", 0) > 25000:
					var ut_val = float(ut.get("total_us", 0)) / 1000.0
					if ut_val >= 16.0:
						if not "Pattern D: frame after reparse >= 16ms" in failures:
							failures.append("Pattern D: frame after reparse %.2fms >= 16ms" % [ut_val])

		var dg_ok_str = "yes" if dirty_gate_ok else "no"
		print("%-10s %-6d %-13.2f %-13.2f %-13.2f %-13s" % [p, raw.size(), med_reify, p95_total, max_total, dg_ok_str])
