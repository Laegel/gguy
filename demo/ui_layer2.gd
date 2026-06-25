extends Node

@export var surface: Node = null

func load_template(template_html: String, base_css: String = "") -> void:
    if surface == null:
        return
    var full_html = "<html><head><style>" + base_css + "</style></head><body>" + template_html + "</body></html>"
    surface.load_html(full_html)

func update(values: Dictionary) -> void:
    if surface == null:
        return
    for id in values:
        surface.set_text(id, str(values[id]))
