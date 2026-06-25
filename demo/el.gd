extends Node

static func el(tag: String, attrs: Dictionary = {}, children: Array = []) -> Dictionary:
    return {"tag": tag, "attrs": attrs, "children": children}

static func text(value) -> Dictionary:
    return {"text": str(value)}

static func div(attrs: Dictionary = {}, children: Array = []) -> Dictionary:
    return el("div", attrs, children)

static func span(attrs: Dictionary = {}, children: Array = []) -> Dictionary:
    return el("span", attrs, children)

static func h1(attrs: Dictionary = {}, children: Array = []) -> Dictionary:
    return el("h1", attrs, children)

static func h2(attrs: Dictionary = {}, children: Array = []) -> Dictionary:
    return el("h2", attrs, children)

static func h3(attrs: Dictionary = {}, children: Array = []) -> Dictionary:
    return el("h3", attrs, children)

static func p(attrs: Dictionary = {}, children: Array = []) -> Dictionary:
    return el("p", attrs, children)

static func button(attrs: Dictionary = {}, children: Array = []) -> Dictionary:
    return el("button", attrs, children)

static func img(attrs: Dictionary = {}) -> Dictionary:
    return el("img", attrs, [])
