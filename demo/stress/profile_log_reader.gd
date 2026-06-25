extends RefCounted

var file: FileAccess = null
var path: String = ""
var cursor: int = 0
var frames: Array[Dictionary] = []
var ready: bool = false

func _init():
	var args = OS.get_cmdline_args()
	for i in args.size():
		if args[i] == "--log-file" and i + 1 < args.size():
			path = args[i + 1]
			break
	if path.is_empty():
		path = "user://godot.log"
	if not FileAccess.file_exists(path):
		printerr("WARN: profile log not found at ", path)

func poll() -> void:
	if not file:
		file = FileAccess.open(path, FileAccess.READ)
		if not file:
			return
		ready = true
	if cursor > 0:
		file.seek(cursor)
	while not file.eof_reached():
		var line: String = file.get_line().strip_edges()
		if line.is_empty():
			continue
		var entry = _parse_line(line)
		if entry:
			frames.append(entry)
	cursor = file.get_position()

func _parse_line(line: String) -> Dictionary:
	if not "[profile]" in line:
		return {}
	if "update_texture:" in line:
		var e = { "type": "update_texture", "raw": line }
		var m = _match(line, "total=(\\d+)")
		if m: e.total_us = int(m)
		m = _match(line, "reify=(\\d+)")
		if m: e.reify_us = int(m)
		m = _match(line, "paint=(\\d+)")
		if m: e.paint_us = int(m)
		m = _match(line, "tex=(\\S+)")
		if m: e.tex = m
		return e
	if "reify:" in line:
		var e = { "type": "reify", "raw": line }
		var m = _match(line, "total=(\\d+)")
		if m: e.reify_total_us = int(m)
		m = _match(line, "scroll=(\\d+)")
		if m: e.scroll_us = int(m)
		m = _match(line, "style=(\\d+)")
		if m: e.style_us = int(m)
		m = _match(line, "construct=(\\d+)")
		if m: e.construct_us = int(m)
		m = _match(line, "deferred=(\\d+)")
		if m: e.deferred_us = int(m)
		m = _match(line, "flush=(\\d+)")
		if m: e.flush_us = int(m)
		m = _match(line, "layout=(\\d+)")
		if m: e.layout_us = int(m)
		return e
	if "paint_scene:" in line:
		var e = { "type": "paint_scene", "raw": line }
		var m = _match(line, "(\\d+)µs")
		if m: e.paint_us = int(m)
		return e
	if "render_scene" in line:
		var e = { "type": "render_scene", "raw": line }
		var m = _match(line, "(\\d+)µs")
		if m: e.render_us = int(m)
		return e
	if "texture_create:" in line:
		var e = { "type": "texture_create", "raw": line }
		var m = _match(line, "(\\d+)µs")
		if m: e.tex_create_us = int(m)
		return e
	return {}

func _match(text: String, pattern: String) -> String:
	var regex = RegEx.new()
	regex.compile(pattern)
	var result = regex.search(text)
	if result:
		return result.get_string(1)
	return ""

func clear() -> void:
	frames.clear()
	cursor = 0

func get_update_texture_frames() -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for f in frames:
		if f.get("type") == "update_texture":
			result.append(f)
	return result

func get_reify_frames() -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for f in frames:
		if f.get("type") == "reify":
			result.append(f)
	return result

func has_all_surfaces_rendered(surface_count: int, within_last_n: int) -> bool:
	var recent = frames.slice(max(0, frames.size() - within_last_n))
	var surfaces_seen: Dictionary = {}
	for f in recent:
		var s = f.get("surface", "")
		if not s.is_empty():
			surfaces_seen[s] = true
		elif f.get("type") in ["update_texture", "reify", "texture_create"]:
			surfaces_seen["_default"] = true
	return surfaces_seen.size() >= surface_count
