use log::info;

use crate::{myshader::MyShader, widgets::player};

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(default)]
pub struct MainApp {
    sh: MyShader,
    player: player::PlayerState,
}

impl MainApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { sh, player } = self;

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
                ui.color_edit_button_rgba_unmultiplied(&mut sh.style.fill);
                ui.label("fill");
                ui.add_space(10.0);
                ui.color_edit_button_rgba_unmultiplied(&mut sh.style.edge);
                ui.label("edge");
            });
            ui.add(
                egui::Slider::new(&mut sh.style.line_width_px, 0.0..=10.0).text("line width (px)"),
            );
            ui.add(
                egui::Slider::new(&mut sh.style.corner_radius_px, 0.0..=50.0)
                    .text("corner radius (px)"),
            );
            ui.add(egui::Slider::new(&mut sh.rect_count, 1..=100).text("Rectangle count"));
            ui.add(*sh); // FIXME: more like a custom paintable then a widget
            ui.add(player::Controller::new(
                &mut self.player,
                &mut sh.time_seconds,
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                ui.hyperlink_to(
                    "try-egui-eframe",
                    "https://github.com/nclack/try-egui-eframe",
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Central Panel");
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        info!("SAVE SETTINGS");
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
