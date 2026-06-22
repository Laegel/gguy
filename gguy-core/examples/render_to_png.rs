use gguy_core::{Document, RenderOutput, RenderThread};
use image::ColorType;

fn render_output(output: &RenderOutput, path: &str) {
    image::save_buffer(
        path,
        output.bytes(),
        output.width(),
        output.height(),
        ColorType::Rgba8,
    )
    .unwrap();
    println!("Wrote {} ({}x{})", path, output.width(), output.height());
}

fn main() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
  body {
    margin: 0;
    padding: 20px;
    background: #1a1a2e;
    color: #e0e0e0;
    font-family: system-ui, sans-serif;
  }
  .container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 600px;
    margin: 0 auto;
  }
  .card {
    background: #16213e;
    border-radius: 12px;
    padding: 20px;
    border: 1px solid #0f3460;
  }
  .card h2 {
    margin: 0 0 8px 0;
    color: #e94560;
    font-size: 24px;
  }
  .card p {
    margin: 0;
    line-height: 1.5;
    color: #a0a0b0;
  }
  .badge {
    display: inline-block;
    background: #e94560;
    color: #fff;
    padding: 4px 12px;
    border-radius: 999px;
    font-size: 14px;
    font-weight: 600;
  }
</style>
</head>
<body>
  <div class="container">
    <div class="card">
      <h2>Hello, Gguy!</h2>
      <p>This is rendered via Blitz → vello → wgpu → CPU readback (threaded).</p>
    </div>
    <div class="card">
      <h2>Flexbox layout</h2>
      <p>Cards are stacked with <code>gap: 16px</code> and centered.</p>
    </div>
    <div>
      <span class="badge">v0.1.0</span>
    </div>
  </div>
</body>
</html>"#;

    let width = 800u32;
    let height = 600u32;
    let scale = 1.0;

    let phys_w = (width as f64 * scale) as u32;
    let phys_h = (height as f64 * scale) as u32;

    let mut doc = Document::new(html, width, height, scale);
    let mut rt = RenderThread::new(phys_w, phys_h);

    doc.reify();
    let scene = doc.paint_scene();
    rt.send_scene(scene, phys_w, phys_h);

    let output = loop {
        if let Some(output) = rt.try_recv() {
            break output;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    };

    render_output(&output, "render_output.png");

    // Second frame: change doc, render again
    doc.reify();
    let scene = doc.paint_scene();
    rt.send_scene(scene, phys_w, phys_h);

    let output = loop {
        if let Some(output) = rt.try_recv() {
            break output;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    };

    render_output(&output, "render_output_2.png");
}
