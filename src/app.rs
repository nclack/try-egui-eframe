use egui::Button;
use log::info;

use crate::widgets::{simple_image::ui::SimpleImage, wavy_rects::ui::WavyRectanglesWithControls};

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(default)]
pub struct MainApp {
    wavy_rectangles: WavyRectanglesWithControls,
    wavy_rectangles2: WavyRectanglesWithControls,
    simple_image: SimpleImage,
    should_display_profiler: bool,
}

impl MainApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        puffin::set_scopes_on(true);
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        app.wavy_rectangles.setup_renderer(cc);
        app.wavy_rectangles2.setup_renderer(cc);
        app.simple_image.setup_renderer(cc);
        app
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        puffin::GlobalProfiler::lock().new_frame();

        let Self {
            wavy_rectangles,
            wavy_rectangles2,
            simple_image,
            should_display_profiler,
        } = self;

        if *should_display_profiler {
            *should_display_profiler = puffin_egui::profiler_window(ctx);
        }

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
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                egui::warn_if_debug_build(ui);
                ui.hyperlink_to(
                    "try-egui-eframe",
                    "https://github.com/nclack/try-egui-eframe",
                );
                if !*should_display_profiler {
                    if ui.add(Button::new("Open Profiler")).clicked() {
                        *should_display_profiler = true;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rounded rectangles");
            ui.columns(2, |columns| {
                columns[0].add(wavy_rectangles);
                columns[1].add(wavy_rectangles2);
            });
            ui.add(simple_image);
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        info!("SAVE SETTINGS");
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
