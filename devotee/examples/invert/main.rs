use devotee::app;
use devotee::app::config;
use devotee::app::context::UpdateContext;
use devotee::app::input::{Keyboard, VirtualKeyCode};
use devotee::app::setup;
use devotee::node::Node;
use devotee::util::vector::Vector;
use devotee::visual::canvas::Canvas;
use devotee::visual::color;
use devotee::visual::prelude::*;
use devotee::visual::UnsafePixel;

fn main() {
    let init_config = setup::Setup::<Config>::new(|_| RootNode::new(), Default::default())
        .with_title("invert")
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

struct RootNode {
    position: Vector<f64>,
    canvas: Canvas<bool>,
}

impl RootNode {
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

impl Node<&mut UpdateContext<Keyboard>, &mut Canvas<Color>> for RootNode {
    fn update(&mut self, update: &mut UpdateContext<Keyboard>) {
        if update.input().just_key_pressed(VirtualKeyCode::Escape) {
            update.shutdown();
        }
        let delta = 16.0 * update.delta().as_secs_f64();
        if update.input().is_key_pressed(VirtualKeyCode::Left) {
            *self.position.x_mut() -= delta;
        }
        if update.input().is_key_pressed(VirtualKeyCode::Right) {
            *self.position.x_mut() += delta;
        }
        if update.input().is_key_pressed(VirtualKeyCode::Up) {
            *self.position.y_mut() -= delta;
        }
        if update.input().is_key_pressed(VirtualKeyCode::Down) {
            *self.position.y_mut() += delta;
        }
    }

    fn render(&self, render: &mut Canvas<Color>) {
        render.clear(Color([0, 0, 0]));
        for x in 8..(render.width() - 8) {
            for y in 8..(render.height() - 8) {
                unsafe {
                    *UnsafePixel::pixel_mut(render, (x, y)) =
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
