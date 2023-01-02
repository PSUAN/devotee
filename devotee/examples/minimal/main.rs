use std::f64::consts::FRAC_PI_3;

use devotee::app;
use devotee::app::config;
use devotee::app::context::Context;
use devotee::app::input::{Keyboard, VirtualKeyCode};
use devotee::app::setup;
use devotee::node::Node;
use devotee::visual::canvas::Canvas;
use devotee::visual::color;
use devotee::visual::prelude::*;

fn main() {
    let init_config = setup::Setup::<Config>::default()
        .with_title("minimal")
        .with_resolution((128, 128))
        .with_scale(2);
    let app = app::App::with_setup(init_config).unwrap();

    app.run();
}

struct Config;

impl config::Config for Config {
    type Node = RootNode;
    type Palette = Color;
    type Converter = Converter;
    type Input = Keyboard;

    fn converter() -> Self::Converter {
        Converter
    }

    fn background_color() -> Self::Palette {
        Color([0, 0, 0])
    }
}

#[derive(Default)]
struct RootNode {
    counter: f64,
}

impl Node<&mut Context<Keyboard>, &mut Canvas<Color>> for RootNode {
    fn update(&mut self, update: &mut Context<Keyboard>) {
        if update.input().just_key_pressed(VirtualKeyCode::Escape) {
            update.shutdown();
        }
        self.counter += update.delta().as_secs_f64();
    }

    fn render(&self, render: &mut Canvas<Color>) {
        let red = (self.counter.sin() * 128.0) as u8 + 127;
        let green = ((self.counter + FRAC_PI_3).sin() * 128.0) as u8 + 127;
        let blue = ((self.counter + 2.0 * FRAC_PI_3).sin() * 128.0) as u8 + 127;

        render.clear(Color([red, green, blue]));

        render.circle((64, 64), 32, paint(Color([255, 255, 255])));
        render.filled_circle((64, 64), 8, paint(Color([255, 255, 255])));
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
