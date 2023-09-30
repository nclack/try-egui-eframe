use std::f32::consts::PI;

use eframe::{egui_wgpu, CreationContext};
use egui::Widget;
use serde::{Deserialize, Serialize};

use crate::oreb::{self, Vertex};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(default)]
pub struct MyShader {
    pub line_width_px: f32,
    pub corner_radius_px: f32,
    pub rect_count: u32,
    pub time_seconds: f32,
    pub line_color: [f32; 4],
    pub fill_color: [f32; 4],
}

impl Default for MyShader {
    fn default() -> Self {
        Self {
            line_width_px: 0.5,
            corner_radius_px: 0.0,
            rect_count: 20,
            time_seconds: 0.0,
            line_color: [1.0, 0.6, 0.1, 0.5],
            fill_color: [0.8, 0.8, 0.8, 0.2],
        }
    }
}

impl MyShader {
    pub fn init<'a>(&self, cc: &'a CreationContext<'a>) {
        let rc = cc.wgpu_render_state.clone().unwrap();
        let painter = oreb::Painter::new(&rc);

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `MyShader` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        rc.renderer.write().callback_resources.insert(painter);
    }
}

impl Widget for &mut MyShader {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::splat(300.0),
            egui::Sense::focusable_noninteractive(),
        );

        ui.painter()
            .add(egui_wgpu::Callback::new_paint_callback(rect, *self));

        response
    }
}

impl egui_wgpu::CallbackTrait for MyShader {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'a>,
        callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        let painter: &oreb::Painter = callback_resources.get().unwrap();
        painter.paint(render_pass);
    }

    fn prepare(
        &self,
        _device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        let painter: &mut oreb::Painter = callback_resources.get_mut().unwrap();
        let (vertices, indices) = encode_geometry(&make_rects(
            self.time_seconds,
            self.rect_count,
            -0.9,
            0.9,
            -0.9,
            0.9,
        ));
        painter.set_geometry(queue, &vertices, &indices);
        painter.set_uniforms(
            queue,
            // TODO: styling?
            &oreb::PainterSettings {
                // raw
                edge: self.line_color,
                fill: self.fill_color,
                line_width_px: self.line_width_px,
                corner_radius_px: self.corner_radius_px,
            },
        );
        Vec::new()
    }

    fn finish_prepare(
        &self,
        _device: &eframe::wgpu::Device,
        _queue: &eframe::wgpu::Queue,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        _callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        Vec::new()
    }
}

struct Rect {
    center: [f32; 2],
    size: [f32; 2],
    orientation_radians: f32,
}

// x0,x1,y0,y1 are the bounds within which the rectangles should be generated.
// They should be in clip space.
fn make_rects(time_seconds: f32, steps: u32, x0: f32, x1: f32, y0: f32, y1: f32) -> Vec<Rect> {
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
