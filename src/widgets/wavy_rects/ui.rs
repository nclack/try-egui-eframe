use std::f32::consts::PI;

use eframe::{egui_wgpu, CreationContext};
use egui::{vec2, Align, Layout, Widget};
use log::info;
use serde::{Deserialize, Serialize};

use crate::widgets::player::{self, PlayerState};

use super::painter::{RectPainter, RectPainterSettings, Vertex};

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(default)]
pub struct WavyRectanglesWithControls {
    wavy_rectangles: WavyRectangles,
    player: PlayerState,
}

impl WavyRectanglesWithControls {
    pub fn setup_renderer<'a>(&mut self, cc: &'a CreationContext<'a>) {
        self.wavy_rectangles.setup_renderer(cc);
    }
}

impl Widget for &mut WavyRectanglesWithControls {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let WavyRectanglesWithControls {
            wavy_rectangles,
            player,
        } = self;

        let w = ui.available_size_before_wrap().min_elem();

        ui.allocate_ui_with_layout(
            vec2(w, w),
            egui::Layout::top_down_justified(egui::Align::Center),
            |ui| {
                ui.horizontal(|ui| {
                    ui.color_edit_button_rgba_unmultiplied(&mut wavy_rectangles.style.fill);
                    ui.label("fill");
                    ui.add_space(10.0);
                    ui.color_edit_button_rgba_unmultiplied(&mut wavy_rectangles.style.edge);
                    ui.label("edge");
                });
                ui.add(
                    egui::Slider::new(&mut wavy_rectangles.style.line_width_px, 0.0..=10.0)
                        .text("line width (px)"),
                );
                ui.add(
                    egui::Slider::new(&mut wavy_rectangles.style.corner_radius_px, 0.0..=50.0)
                        .text("corner radius (px)"),
                );
                ui.add(
                    egui::Slider::new(&mut wavy_rectangles.rect_count, 1..=100)
                        .text("Rectangle count"),
                );
                ui.allocate_ui_with_layout(
                    ui.available_size_before_wrap(),
                    Layout::bottom_up(Align::Min),
                    |ui| {
                        ui.add(player::Controller::new(
                            player,
                            &mut wavy_rectangles.time_seconds,
                        ));
                        ui.add(*wavy_rectangles);
                    },
                );
            },
        )
        .response
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(default)]
pub struct WavyRectangles {
    pub rect_count: u32,
    pub time_seconds: f32,
    pub style: RectPainterSettings,

    #[serde(skip)]
    id: Option<usize>,
}

impl Default for WavyRectangles {
    fn default() -> Self {
        Self {
            rect_count: 20,
            time_seconds: 0.0,
            style: Default::default(),
            id: None,
        }
    }
}

impl WavyRectangles {
    pub fn setup_renderer<'a>(&mut self, cc: &'a CreationContext<'a>) {
        let rc = cc.wgpu_render_state.clone().unwrap();
        let painter = RectPainter::new(&rc);

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `MyShader` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        // rc.renderer.write().callback_resources.insert(painter);

        let mut renderer = rc.renderer.write();
        let e = renderer
            .callback_resources
            .entry::<Vec<RectPainter>>()
            .or_insert(Vec::new());
        self.id = Some(e.len());
        e.push(painter);
    }
}

impl Widget for WavyRectangles {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let w = ui.available_size_before_wrap().min_elem();

        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::splat(w),
            egui::Sense::focusable_noninteractive(),
        );

        ui.painter()
            .add(egui_wgpu::Callback::new_paint_callback(rect, self));

        response
    }
}

impl egui_wgpu::CallbackTrait for WavyRectangles {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'a>,
        callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        puffin::profile_function!();
        if let Some(id) = self.id {
            let painter: &RectPainter =
                callback_resources.get::<Vec<_>>().unwrap().get(id).unwrap();
            painter.paint(render_pass);
        }
    }

    fn prepare(
        &self,
        _device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        puffin::profile_function!();
        if let Some(id) = self.id {
            let painter: &mut RectPainter = callback_resources
                .get_mut::<Vec<_>>()
                .unwrap()
                .get_mut(id)
                .unwrap();
            let (vertices, indices) = encode_geometry(&make_rects(
                self.time_seconds,
                5.0,
                self.rect_count,
                -0.9,
                0.9,
                -0.9,
                0.9,
            ));
            painter.set_geometry(queue, &vertices, &indices);
            painter.set_uniforms(queue, &self.style);
        }
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
fn make_rects(
    time_seconds: f32,
    scale: f32,
    steps: u32,
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
) -> Vec<Rect> {
    puffin::profile_function!();
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
            let w = scale * sz; // + 1.0 * sz * (2.0 * PI * time_seconds / 0.5).cos();
            let h = scale * sz; // + 1.0 * sz * (2.0 * PI * (0.3 + time_seconds / 3.0)).cos();
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
    puffin::profile_function!();
    fn mk_vertices(rect: &Rect) -> [Vertex; 3] {
        let [cx, cy] = rect.center;
        let [half_w, half_h] = rect.size.map(|e| 0.5 * e);
        let side = half_h + half_w;
        let (s, c) = rect.orientation_radians.sin_cos();

        // The rectangle lies on two sides of the triangle
        // FIXME: need a 1 px padding on those sides for anti-aliasing.

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
