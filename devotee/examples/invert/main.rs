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
use devotee::visual::UnsafePixel;

fn main() {
    let init_config = setup::Setup::<Config>::new(
        Canvas::with_resolution(Color([0, 0, 0]), 128, 128),
        Default::default(),
        |_| Invert::new(),
    )
    .with_title("invert")
    .with_scale(2);
    let app = app::App::with_setup(init_config).unwrap();

    app.run();
}

struct Config;

impl config::Config for Config {
    type Root = Invert;
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

struct Invert {
    position: Vector<f64>,
    canvas: Canvas<bool>,
}

impl Invert {
    fn new() -> Self {
        let position = Vector::new(12.0, 12.0);
        let mut canvas = Canvas::with_resolution(false, 32, 32);

        canvas.line((0, 0), (31, 0), paint(true));
        canvas.line((0, 0), (0, 31), paint(true));
        canvas.line((0, 31), (31, 31), paint(true));
        canvas.line((31, 0), (31, 31), paint(true));
        canvas.line((0, 0), (31, 31), paint(true));
        canvas.line((31, 0), (0, 31), paint(true));

        let mut counter = 0;
        canvas.filled_rect((4, 4), (32 - 4, 32 - 4), move |_, _, _| {
            counter += 1;
            counter % 5 == 0 || counter % 7 == 0
        });

        Self { position, canvas }
    }
}

impl Root<Config> for Invert {
    fn update(&mut self, update: &mut Context<Config>) {
        if update.input().keys().just_pressed(VirtualKeyCode::Escape) {
            update.shutdown();
        }
        let delta = 16.0 * update.delta().as_secs_f64();
        if update.input().keys().is_pressed(VirtualKeyCode::Left) {
            *self.position.x_mut() -= delta;
        }
        if update.input().keys().is_pressed(VirtualKeyCode::Right) {
            *self.position.x_mut() += delta;
        }
        if update.input().keys().is_pressed(VirtualKeyCode::Up) {
            *self.position.y_mut() -= delta;
        }
        if update.input().keys().is_pressed(VirtualKeyCode::Down) {
            *self.position.y_mut() += delta;
        }
    }

    fn render(&self, render: &mut Canvas<Color>) {
        render.clear(Color([0, 0, 0]));
        for x in 8..(render.width() - 8) {
            for y in 8..(render.height() - 8) {
                unsafe {
                    *UnsafePixel::pixel_mut_unsafe(render, (x, y)) =
                        Color([2 * x as u8, 2 * y as u8, 0x00]);
                }
            }
        }

        let (x, y) = (self.position.x() as i32, self.position.y() as i32);
        render.image((x, y), &self.canvas, |_, _, value, _, _, invert| {
            if invert {
                Color([0xff - value.0[0], 0xff - value.0[1], 0xff - value.0[2]])
            } else {
                value
            }
        });
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
