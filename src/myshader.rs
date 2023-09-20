use std::{f32::consts::PI, sync::Arc};

use eframe::{CreationContext, egui_wgpu::CallbackFn};
use egui::{PaintCallback, Widget};

use crate::oreb::{self, Vertex};

pub struct MyShader {}

impl MyShader {
    pub fn new<'a>(cc: &'a CreationContext<'a>) -> Option<Self> {
        let rc = cc.wgpu_render_state.clone()?;
        let painter = oreb::Painter::new(&rc);

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `MyShader` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        rc.renderer.write().paint_callback_resources.insert(painter);
        Some(Self {})
    }
}

impl Widget for &mut MyShader {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::splat(300.0),
            egui::Sense::focusable_noninteractive(),
        );

        let callback = CallbackFn::new()
            .prepare(move |_device, queue, _encoder, resources| {
                let painter: &mut oreb::Painter = resources.get_mut().unwrap();
                let time_seconds = 0.0; // TODO: figure out animation
                let (vertices, indices) =
                    encode_geometry(&make_rects(time_seconds, -0.9, 0.9, -0.9, 0.9));
                painter.set_geometry(queue, &vertices, &indices);
                painter.set_uniforms(
                    queue,
                    // TODO: styling?
                    &oreb::PainterSettings {
                        // raw
                        edge: [1.0, 0.6, 0.1, 0.5],
                        fill: [0.8, 0.8, 0.8, 0.2],
                        line_width_px: 0.5,
                        corner_radius_px: 6.0,
                    },
                );
                Vec::new()
            })
            .paint(move |_info, render_pass, resources| {
                let painter: &oreb::Painter = resources.get().unwrap();
                painter.paint(render_pass);
            });

        ui.painter().add(PaintCallback {
            rect,
            callback: Arc::new(callback),
        });

        response
    }
}

struct Rect {
    center: [f32; 2],
    size: [f32; 2],
    orientation_radians: f32,
}

// x0,x1,y0,y1 are the bounds within which the rectangles should be generated.
// They should be in clip space.
fn make_rects(time_seconds: f32, x0: f32, x1: f32, y0: f32, y1: f32) -> Vec<Rect> {
    let steps = 100;
    let dx = (x1 - x0) / (steps + 1) as f32;
    let dy = y1 - y0;
    let sz = dx.max(0.1);
    (0..steps)
        .map(|i| {
            let is_odd = (i & 1) == 1;
            let i = i as f32;
            let ph = 2.0 * PI * i / (steps + 1) as f32;
            let cx = x0 + dx * (i + 0.5);
            let cy = y0 + 0.5 * dy * (1.0 + (ph + 2.0 * PI * time_seconds / 7.0).cos());
            let w = 3.0 * sz; // + 1.0 * sz * (2.0 * PI * time_seconds / 0.5).cos();
            let h = 3.0 * sz; // + 1.0 * sz * (2.0 * PI * (0.3 + time_seconds / 3.0)).cos();
            let th = (2.0 * PI * time_seconds / 7.0) * if is_odd { 1.0 } else { -1.0 };
            Rect {
                center: [cx, cy],
                size: [w, h],
                orientation_radians: th,
            }
        })
        .collect()
}

fn encode_geometry(rects: &[Rect]) -> (Vec<Vertex>, Vec<u32>) {
    fn mk_vertices(rect: &Rect) -> [Vertex; 3] {
        let [cx, cy] = rect.center;
        let [half_w, half_h] = rect.size.map(|e| 0.5 * e);
        let side = half_h + half_w;
        let (s, c) = rect.orientation_radians.sin_cos();

        // create an isosceles right triangle within which the rect will be painted
        // center is at uv: [0,0]
        // rect's [w,h] in uv coords is [1,1]
        [
            // top-left
            Vertex {
                xyz: [-half_w, -half_h, 0.0],
                uv: [-0.5, -0.5],
            },
            // bottom-right
            Vertex {
                xyz: [2.0 * half_h - half_w, -half_h, 0.0],
                uv: [-0.5 + side / half_h, -0.5],
            },
            // bottom-left
            Vertex {
                xyz: [-half_w, 2.0 * half_w - half_h, 0.0],
                uv: [-0.5, -0.5 + side / half_w],
            },
        ]
            .map(|mut v| {
                // rotate about (0,0) by theta
                // then translate
                v.xyz[0] += half_w * 0.5;
                v.xyz[1] += half_h * 0.5;
                let x = v.xyz[0] * c - v.xyz[1] * s;
                let y = v.xyz[0] * s + v.xyz[1] * c;
                v.xyz[0] = x + cx;
                v.xyz[1] = y + cy;
                v
            })
    }

    let verts = rects
        .into_iter()
        .map(|r| mk_vertices(r))
        .flatten()
        .collect();
    let idxs = (0..3 * rects.len() as u32).collect();
    (verts, idxs)
}
