extends Control

@onready var gguy_surface = $GguySurface
@onready var texture_rect = %TextureRect
var texture_set = false

func _ready():
    var html = """
    <html style="height:100%">
    <body style="height:100%;margin:0"><div style="display:flex;flex-direction:column;align-items:center;justify-content:center;width:100%;height:100%;background:#1a1a2e;font-family:sans-serif">
  <h1 id="main" style="color:#e94560;font-size:36px;margin-bottom:8px">Gguy Engine</h1>
  <p style="color:rgba(255,255,255,0.7);font-size:14px;margin-bottom:24px">HTML/CSS UI · Blitz + wgpu → Godot 4</p>
  <div style="display:flex;gap:16px;flex-wrap:wrap;justify-content:center;padding:0 32px">
    <div style="background:#16213e;padding:20px;border-radius:12px;border:1px solid #0f3460;width:200px">
      <h3 style="color:#e94560;margin:0 0 8px 0">Flexbox</h3>
      <p style="color:#aaa;margin:0;font-size:13px">Full flexbox layout support via Blitz + Taffy</p>
    </div>
    <div style="background:#16213e;padding:20px;border-radius:12px;border:1px solid #0f3460;width:200px">
      <h3 style="color:#e94560;margin:0 0 8px 0">System Fonts</h3>
      <p style="color:#aaa;margin:0;font-size:13px">Native system font rendering</p>
    </div>
    <div style="background:#16213e;padding:20px;border-radius:12px;border:1px solid #0f3460;width:200px">
      <h3 style="color:#e94560;margin:0 0 8px 0">Godot Texture</h3>
      <p style="color:#aaa;margin:0;font-size:13px">Rendered via wgpu, uploaded to ImageTexture</p>
    </div>
  </div>

</div><style>#main:hover{color: yellow !important}</style></body>
</html>"""
    gguy_surface.set_size(1152.0, 648.0)
    gguy_surface.load_html(html)


func _process(_delta):
    # if texture_set:
    #     return
    var tex = gguy_surface.get_texture()
    if tex.get_width() > 0:
        texture_rect.texture = tex
        texture_rect.custom_minimum_size = Vector2(1152, 648)
        texture_set = true
