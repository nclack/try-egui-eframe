use log::info;

use crate::{myimage::MyImage, myshader::MyShader};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct MainApp {
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,

    #[serde(skip)]
    im: MyImage,

    sh: MyShader,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            value: 2.7,
            im: MyImage::default(),
            sh: MyShader::default(),
        }
    }
}

impl MainApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let app: Self = if let Some(storage) = cc.storage {
            info!("LOAD SETTINGS");
            dbg!(eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default())
        } else {
            Default::default()
        };
        app.sh.init(cc);
        app
    }
}

impl eframe::App for MainApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { value: _, im, sh } = self;

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Rounded rectangles");

            ui.horizontal(|ui| {
                ui.color_edit_button_rgba_unmultiplied(&mut sh.fill_color);
                ui.label("fill");
                ui.add_space(10.0);
                ui.color_edit_button_rgba_unmultiplied(&mut sh.line_color);
                ui.label("edge");
            });
            ui.add(egui::Slider::new(&mut sh.line_width_px, 0.0..=10.0).text("line width (px)"));
            ui.add(
                egui::Slider::new(&mut sh.corner_radius_px, 0.0..=50.0).text("corner radius (px)"),
            );
            ui.add(egui::Slider::new(&mut sh.time_seconds, -1.0..=1.0).text("time (s)"));
            ui.add(egui::Slider::new(&mut sh.rect_count, 1..=100).text("Rectangle count"));
            ui.add(sh);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                ui.hyperlink_to(
                    "try-egui-eframe",
                    "https://github.com/nclack/try-egui-eframe",
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(im);
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        info!("SAVE SETTINGS");
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
