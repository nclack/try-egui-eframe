use eframe::CreationContext;
use egui::{load::SizedTexture, Image, ImageSource, Sense, Vec2, Widget};
use log::info;
use serde::{Deserialize, Serialize};

use super::painter::Settings;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SimpleImage {
    pub style: Settings,

    #[serde(skip)]
    target: Option<SizedTexture>,

    #[serde(skip)]
    painter: Option<super::painter::Painter>,
}

impl SimpleImage {
    pub fn setup_renderer<'s>(&mut self, cc: &'s CreationContext<'s>) {
        let rc = cc.wgpu_render_state.clone().unwrap();
        let painter = super::painter::Painter::new(&rc, 640, 480).unwrap();

        self.target = {
            let tid = rc.renderer.write().register_native_texture(
                &rc.device,
                &painter.create_texture_view(),
                eframe::wgpu::FilterMode::Nearest,
            );
            Some(SizedTexture::from((tid, Vec2::new(640.0, 480.0))))
        };

        painter.update(&rc.queue, &self.style);
        painter.oneshot(&rc);

        self.painter = Some(painter);
        // rc.renderer.write().callback_resources.insert(painter);
    }
}

impl Widget for &mut SimpleImage {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label("before simple image");
        let r = if let Some(target) = self.target {
            info!("HERE");
            ui.add(Image::new(ImageSource::Texture(target)))
        } else {
            let (_, response) =
                ui.allocate_exact_size(Vec2::new(640.0, 480.0), Sense::focusable_noninteractive());
            response
        };
        ui.label("after simple image");
        r
    }
}
