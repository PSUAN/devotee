use devotee::app;
use devotee::app::config;
use devotee::app::context::Context;
use devotee::app::input::key_mouse::{KeyMouse, VirtualKeyCode};
use devotee::app::root::Root;
use devotee::app::setup;
use devotee::util::vector::Vector;
use devotee::visual::canvas::Canvas;
use devotee::visual::color;
use devotee::visual::prelude::*;

const BOX_BOUNDARIES: (i32, i32) = (16, 128 - 16);
const INTERNAL_RADIUS: i32 = 8;

fn main() {
    let init_config = setup::Setup::<Config>::new(
        Canvas::with_resolution(Default::default(), 128, 128),
        Default::default(),
        |_| Default::default(),
    )
    .with_title("minimal")
    .with_scale(2);
    let app = app::App::with_setup(init_config).unwrap();

    app.run();
}

struct Config;

impl config::Config for Config {
    type Root = Minial;
    type Converter = Converter;
    type Input = KeyMouse;
    type RenderTarget = Canvas<Color>;

    fn converter() -> Self::Converter {
        Converter
    }

    fn background_color() -> Color {
        Color([0, 0, 0])
    }
}

#[derive(Default)]
struct Minial {
    position: Vector<i32>,
}

impl Root<Config> for Minial {
    fn update(&mut self, update: &mut Context<Config>) {
        if update.input().keys().just_pressed(VirtualKeyCode::Escape) {
            update.shutdown();
        }

        if let Some(pos) = update.input().mouse().position() {
            *self.position.x_mut() = pos.x().clamp(
                BOX_BOUNDARIES.0 + INTERNAL_RADIUS,
                BOX_BOUNDARIES.1 - INTERNAL_RADIUS - 1,
            );
            *self.position.y_mut() = pos.y().clamp(
                BOX_BOUNDARIES.0 + INTERNAL_RADIUS,
                BOX_BOUNDARIES.1 - INTERNAL_RADIUS - 1,
            );
        }
    }

    fn render(&self, render: &mut Canvas<Color>) {
        render.clear(Color([0x00, 0x00, 0x80]));

        render.rect(
            (BOX_BOUNDARIES.0, BOX_BOUNDARIES.0),
            (BOX_BOUNDARIES.1, BOX_BOUNDARIES.1),
            paint(Color([0xff, 0xff, 0xff])),
        );
        render.filled_circle(
            self.position,
            INTERNAL_RADIUS,
            paint(Color([0x80, 0x80, 0x80])),
        );
    }
}

#[derive(Clone, Copy, Default)]
struct Color([u8; 3]);

struct Converter;

impl color::Converter for Converter {
    type Palette = Color;
    fn convert(&self, color: &Self::Palette) -> [u8; 4] {
        [color.0[0], color.0[1], color.0[2], 0xff]
    }
}
