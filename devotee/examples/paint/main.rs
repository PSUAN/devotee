use std::f64::consts;
use std::time::Duration;

use devotee::app::context::Context;
use devotee::app::input::key_mouse::{KeyMouse, MouseButton, VirtualKeyCode};
use devotee::app::{self, config, setup};
use devotee::node::Node;
use devotee::util::vector::Vector;
use devotee::visual::sprite::Sprite;
use devotee::visual::{color, stamp, Draw, Image, Pixel};

fn main() {
    let init_config = setup::Setup::<Config>::new(
        Sprite::with_color(Palette { value: 0.0 }),
        Default::default(),
        |_| Default::default(),
    )
    .with_update_delay(Duration::from_secs_f64(1.0 / 60.0))
    .with_title("mouse")
    .with_fullscreen(true);
    let app = app::App::with_setup(init_config).unwrap();

    app.run();
}

struct Config;

impl config::Config for Config {
    type Node = PaintNode;

    type Converter = Converter;

    type Input = KeyMouse;

    type RenderTarget = Sprite<Palette, 64, 64>;

    fn converter() -> Self::Converter {
        Converter { hue: 0.0 }
    }

    fn background_color() -> <Self::RenderTarget as devotee::visual::Draw>::Pixel {
        Palette { value: 0.0 }
    }
}

#[derive(Default)]
struct PaintNode {
    droplets: Vec<Droplet>,
    canvas: Sprite<Palette, 64, 64>,
    cursor: Option<Vector<i32>>,
}

impl Node<&mut Context<Config>, &mut Sprite<Palette, 64, 64>> for PaintNode {
    fn update(&mut self, update: &mut Context<Config>) {
        if update.input().keys().just_pressed(VirtualKeyCode::Escape) {
            update.shutdown()
        }

        self.cursor = update.input().mouse().position();
        if update.input().mouse().is_pressed(MouseButton::Left) {
            if let Some(position) = self.cursor {
                self.droplets.push(Droplet {
                    position,
                    wetness_left: 2.0,
                    delay_left: 0.5,
                });
                if let Some(pixel) = self.canvas.pixel_mut(position) {
                    pixel.value += 0.1;
                }
            }
        }

        let delta = update.delta().as_secs_f64();

        update.converter_mut().hue += delta;

        for droplet in self.droplets.iter_mut() {
            if let Some(pixel) = self.canvas.pixel_mut(droplet.position) {
                pixel.value += 0.1 * delta;
            }
            droplet.delay_left -= delta;
            if droplet.delay_left < 0.0 {
                *droplet.position.y_mut() += 1;
                droplet.delay_left += 0.5;
            }

            droplet.wetness_left -= delta;
        }

        self.droplets.retain(|droplet| droplet.wetness_left > 0.0);
    }

    fn render(&self, render: &mut Sprite<Palette, 64, 64>) {
        render.clear(Palette { value: 0.25 });

        render.image((0, 0), &self.canvas, stamp());

        if let Some(cursor) = self.cursor {
            if let Some(pixel) = render.pixel_mut(cursor) {
                pixel.value = 1.0;
            }
        }
    }
}

struct Droplet {
    position: Vector<i32>,
    wetness_left: f64,
    delay_left: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
struct Palette {
    value: f64,
}

struct Converter {
    hue: f64,
}

impl color::Converter for Converter {
    type Palette = Palette;

    fn convert(&self, color: &Self::Palette) -> [u8; 4] {
        let amplitude = color.value.clamp(0.0, 1.0);
        let red = 0.5 * self.hue.cos() + 0.5;
        let green = 0.5 * (self.hue + 2.0 * consts::FRAC_PI_3).cos() + 0.5;
        let blue = 0.5 * (self.hue - 2.0 * consts::FRAC_PI_3).cos() + 0.5;

        [
            (red * amplitude * 255.0) as u8,
            (green * amplitude * 255.0) as u8,
            (blue * amplitude * 255.0) as u8,
            0xff,
        ]
    }
}
