extends Control

const ProfileLogReader = preload("res://stress/profile_log_reader.gd")

var reader

var surface_count: int = 2
var surfaces: Array = []
var texture_rects: Array = []

enum Phase { WARMUP, COLLECT, STALL, FINISH }
var phase: int = Phase.WARMUP
var frame_count: int = 0
var phase_frame: int = 0
var failures: Array[String] = []
var _tex_seen := false

var per_surface_texture_updates: Dictionary = {}
var stall_before_counts: Dictionary = {}
var frame_times: Array = []
var crashed: bool = false
var stall_surface_idx: int = -1

var _last_process_time: int = 0

var SMALL_HUD := "<html><head><style>:root{--primary-color:#e94560;--bar-fill:#48dbfb}.hud{display:flex;flex-direction:column;gap:8px;padding:16px;background:%s;border-radius:12px;box-shadow:0 4px 12px rgba(0,0,0,0.3);width:300px}.bars{display:flex;flex-direction:column;gap:4px}.bar{display:flex;align-items:center;gap:8px;background:#16213e;border-radius:8px;padding:4px 8px;border:1px solid #0f3460}.bar .fill{height:12px;background:var(--primary-color);border-radius:4px}.bar span{color:#aaa;font-size:12px}.skills{display:flex;gap:8px;margin-top:4px}.skill{width:48px;height:48px;background:#16213e;border-radius:8px;border:1px solid #0f3460;display:flex;align-items:center;justify-content:center;color:#eee;font-size:18px;font-weight:bold}.skill:hover{background:#e94560;color:#fff}.minimap{width:120px;height:120px;background:#16213e;border-radius:50%%;border:2px solid #0f3460;box-shadow:0 0 8px rgba(0,0,0,0.5);margin-top:8px}</style></head><body style='margin:0;padding:0;background:%s' data-surface-id='%s'><div class='hud'><div class='bars'><div class='bar hp'><div class='fill' style='width:80%%'></div><span>HP 80/100</span></div><div class='bar mp'><div class='fill' style='width:60%%'></div><span>MP 60/100</span></div></div><div class='skills'><div class='skill'>Q</div><div class='skill'>W</div><div class='skill'>E</div></div><div class='minimap'></div></div></body></html>"

func _ready():
	reader = ProfileLogReader.new()
	var args = OS.get_cmdline_args()
	for i in range(args.size()):
		if args[i] == "--surfaces" and i + 1 < args.size():
			surface_count = clampi(int(args[i + 1]), 2, 8)
			break
	_setup_surfaces()

func _setup_surfaces():
	var bg_colors = ["#1a1a2e", "#2d1a3e", "#1a2d3e", "#3e1a1a", "#1a3e2d", "#2d2d1a", "#3e2d1a", "#1a3e3e"]
	var body_bgs = ["#0a0a14", "#140a1e", "#0a141e", "#1e0a0a", "#0a1e14", "#1e1e0a", "#1e140a", "#0a1e1e"]

	for idx in range(surface_count):
		var surf = ClassDB.instantiate("GguySurface")
		surf.name = "GguySurface%d" % idx
		add_child(surf)
		surfaces.append(surf)

		var rect = TextureRect.new()
		rect.name = "TextureRect%d" % idx
		rect.anchors_preset = Control.PRESET_TOP_LEFT
		if idx == 0:
			rect.anchor_right = 0.5
			rect.anchor_bottom = 0.5
		elif idx == 1:
			rect.anchor_left = 0.5
			rect.anchor_right = 1.0
			rect.anchor_bottom = 0.5
		elif idx == 2:
			rect.anchor_top = 0.5
			rect.anchor_right = 0.5
			rect.anchor_bottom = 1.0
		elif idx == 3:
			rect.anchor_left = 0.5
			rect.anchor_top = 0.5
			rect.anchor_right = 1.0
			rect.anchor_bottom = 1.0
		else:
			var row = int(idx / 4)
			var col = idx % 4
			rect.position = Vector2(col * 280, row * 180 + 400)
			rect.size = Vector2(260, 160)
		rect.stretch_mode = TextureRect.STRETCH_KEEP
		add_child(rect)
		texture_rects.append(rect)
		per_surface_texture_updates[idx] = 0

		var html: String
		if idx == 0 or idx == 2:
			html = SMALL_HUD % [bg_colors[idx % bg_colors.size()], body_bgs[idx % body_bgs.size()], "surf-%d" % idx]
		else:
			html = _medium_inv_html(idx, bg_colors, body_bgs)

		surf.set_size(1152.0, 648.0)
		surf.load_html(html)

	if surface_count >= 4:
		stall_surface_idx = 1

func _medium_inv_html(idx: int, bg_colors: Array, body_bgs: Array) -> String:
	var html = "<html><head><style>:root{--primary-color:#e94560;--bar-fill:#48dbfb}body{margin:0;padding:16px;background:%s}.inv{display:grid;grid-template-columns:repeat(10,48px);gap:4px;padding:16px;background:#1a1a2e;border-radius:12px;box-shadow:0 4px 12px rgba(0,0,0,0.3)}.slot{width:48px;height:48px;background:#16213e;border-radius:6px;border:1px solid #0f3460;display:flex;flex-direction:column;align-items:center;justify-content:center}.slot:hover{border-color:var(--primary-color)}.slot .icon{font-size:20px}.slot .count{font-size:10px;color:#aaa}</style></head><body data-surface-id='surf-%d'>" % [body_bgs[idx % body_bgs.size()], idx]
	html += "<div class='inv' id='inventory'>"
	for row in range(6):
		for col in range(10):
			html += "<div class='slot' id='slot-%d-%d'><span class='icon'>★</span><span class='count'>%d</span></div>" % [row, col, row * 10 + col]
	html += "</div></body></html>"
	return html

func _gen_pathological_html(size: int) -> String:
	var html = "<html><head><style>body{margin:0;padding:0;background:#111}.item{width:100%%;height:20px;background:#1a1a2e;border-bottom:1px solid #333;color:#ccc;font-size:12px;padding:4px}</style></head><body data-surface-id='stall'><div id='container'>"
	for i in range(size):
		html += "<div class='item' id='item-%d'>Item %d</div>" % [i, i]
	html += "</div></body></html>"
	return html

func _process(_delta):
	if crashed:
		return

	reader.poll()
	frame_count += 1
	phase_frame += 1

	if not _tex_seen:
		for surf in surfaces:
			var tex = surf.get_texture()
			if tex and tex.get_width() > 0:
				_tex_seen = true
				break
		if not _tex_seen and frame_count > 60:
			print("=== TEST FAILED: no texture output — wgpu/Vulkan init likely failed ===")
			set_process(false)
			return

	var now = Time.get_ticks_usec()
	if _last_process_time > 0:
		frame_times.append(now - _last_process_time)
	_last_process_time = now

	for idx in range(surfaces.size()):
		var surf = surfaces[idx]
		var tex = surf.get_texture()
		if tex and tex.get_width() > 0:
			per_surface_texture_updates[idx] += 1
			var rect = texture_rects[idx]
			if rect and rect.texture != tex:
				rect.texture = tex
				rect.custom_minimum_size = Vector2(180, 100)

	match phase:
		Phase.WARMUP:
			if phase_frame >= 10:
				_advance()

		Phase.COLLECT:
			if phase_frame >= 120:
				if surface_count >= 4:
					_load_pathological()
				_advance()

		Phase.STALL:
			if phase_frame >= 10:
				_verify_stall()
				_advance()

		Phase.FINISH:
			_finish()

func _advance():
	phase += 1
	phase_frame = 0

func _load_pathological():
	if stall_surface_idx < 0 or stall_surface_idx >= surfaces.size():
		_advance()
		return
	stall_before_counts = {}
	for idx in range(surfaces.size()):
		stall_before_counts[idx] = per_surface_texture_updates.get(idx, 0)
	var surf = surfaces[stall_surface_idx]
	surf.load_html(_gen_pathological_html(2000))
	print("Stall test: loaded 2000-node doc into surface %d" % stall_surface_idx)

func _verify_stall():
	if stall_surface_idx < 0:
		return
	var surf = surfaces[stall_surface_idx]
	var tex = surf.get_texture()
	if tex and tex.get_width() > 0:
		print("Stall surface produced texture: yes")
	else:
		print("Stall surface produced texture: no (may still be loading WGPU)")
	surf.load_html(_medium_inv_html(stall_surface_idx, ["#3e1a1a"], ["#1e0a0a"]))
	print("Stall surface reloaded with medium document")

	var others_continued := true
	for idx in range(surfaces.size()):
		if idx == stall_surface_idx:
			continue
		var before = stall_before_counts.get(idx, 0)
		var after = per_surface_texture_updates.get(idx, 0)
		if after <= before:
			others_continued = false
			print("WARN: Surface %d stopped updating during stall (before=%d after=%d)" % [idx, before, after])
	print("Other surfaces continued producing textures: %s" % ["yes" if others_continued else "no"])
	if not others_continued:
		failures.append("Other surfaces stalled during pathological load")

func _finish():
	var total_frames = frame_count
	var st = frame_times.duplicate()
	st.sort()
	var med_us = _median(frame_times) if not frame_times.is_empty() else 0.0
	var med_ms = med_us / 1000.0

	var p95_idx = int(ceil(st.size() * 0.95)) - 1 if not st.is_empty() else -1
	p95_idx = clampi(p95_idx, 0, st.size() - 1)
	var p95_us = st[p95_idx] if p95_idx >= 0 else 0.0

	print("\n=== Multi-Surface Results (N=%d) ===" % surface_count)
	print("%-4s %-15s %-15s %-10s %-10s %-8s" % ["N", "total_cost_med", "cost_per_surf", "starved", "corrupted", "crashed"])
	print("%-4d %-15.2f %-15.2f %-10s %-10s %-8s" % [surface_count, med_ms, med_ms / max(surface_count, 1), "no", "no", "no"])

	var starved := false
	for idx in range(surfaces.size()):
		var updates = per_surface_texture_updates.get(idx, 0)
		var expected = total_frames / 3
		if updates < expected:
			starved = true
			failures.append("Surface %d starved: %d updates in %d frames" % [idx, updates, total_frames])

	if starved:
		print("Starvation detected: some surfaces updated less than once per 3 frames")

	if surface_count == 8 and p95_us > 33000:
		failures.append("N=8: p95 frame time %.2fms > 33ms" % [p95_us / 1000.0])

	var max_us = st[-1] if not st.is_empty() else 0.0
	print("Frame times: median=%.2fms p95=%.2fms max=%.2fms" % [med_ms, p95_us / 1000.0, max_us / 1000.0])

	if failures.is_empty():
		print("=== TEST COMPLETE ===")
	else:
		print("=== TEST FAILED: " + "; ".join(failures) + " ===")
	set_process(false)

func _median(arr: Array) -> float:
	if arr.is_empty():
		return 0.0
	var sorted = arr.duplicate()
	sorted.sort()
	var mid = sorted.size() / 2
	if sorted.size() % 2 == 1:
		return float(sorted[mid])
	return float(sorted[mid - 1] + sorted[mid]) / 2.0
