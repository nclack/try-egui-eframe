use egui::{include_image, vec2, ImageButton, NumExt, ProgressBar, Response, Sense, Widget};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct PlayerState {
    is_playing: bool,

    #[serde(skip)]
    last_pause_time: Option<f64>,
}

impl PlayerState {
    // Progress in seconds adjusted for the last pause time
    pub fn progress(&self, current_time: f64) -> f64 {
        let Self {
            is_playing,
            last_pause_time,
        } = self;
        if *is_playing {
            current_time - (last_pause_time.unwrap_or(0.0))
        } else {
            last_pause_time.unwrap_or(0.0)
        }
    }
}

pub struct Controller<'a, 'b> {
    state: &'a mut PlayerState,
    progress_seconds: &'b mut f32,
}

impl<'a, 'b> Controller<'a, 'b> {
    pub fn new(state: &'a mut PlayerState, progress_seconds: &'b mut f32) -> Self {
        Self {
            state,
            progress_seconds,
        }
    }
}

impl<'a, 'b> Widget for Controller<'a, 'b> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let current_time = ui.input(|i| i.time);
        *self.progress_seconds = self.state.progress(current_time) as _;
        let secs = *self.progress_seconds;

        let PlayerState {
            is_playing,
            last_pause_time,
        } = self.state;

        let desired_width = ui.available_size_before_wrap().x.at_least(96.0);
        let height = ui.spacing().interact_size.y;
        let (_outer_rect, response) =
            ui.allocate_exact_size(vec2(desired_width, height), Sense::hover());

        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    vec2(height, height),
                    ImageButton::new(if *is_playing {
                        include_image!("assets/pause-solid.svg")
                    } else {
                        include_image!("assets/play-solid.svg")
                    }),
                )
                .clicked()
            {
                *is_playing = !*is_playing;
                *last_pause_time = Some(current_time - (last_pause_time.unwrap_or(0.0)));
            }
            ui.add(
                ProgressBar::new((secs / 10.0).fract() as f32)
                    .animate(*is_playing)
                    .text(format!("{secs:2.2} s")),
            );
        });

        response
    }
}
