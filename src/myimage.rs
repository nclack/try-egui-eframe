use egui::{Response, Ui, Widget};

#[derive(Default)]
pub struct MyImage {
    texture: Option<egui::TextureHandle>,
}

impl Widget for &mut MyImage {
    fn ui(self, ui: &mut Ui) -> Response {
        let texture: &egui::TextureHandle = self.texture.get_or_insert_with(|| {
            ui.ctx()
                .load_texture("my-image", egui::ColorImage::example(), Default::default())
        });

        ui.image(texture)
    }
}

impl std::fmt::Debug for MyImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyImage").finish()
    }
}
