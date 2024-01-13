use std::f64::consts;

use devotee::app;
use devotee::app::config;
use devotee::app::context::Context;
use devotee::app::input::key_mouse::{KeyMouse, VirtualKeyCode};
use devotee::app::root::Root;
use devotee::app::setup;
use devotee::util::vector::Vector;
use devotee::visual::color;
use devotee::visual::prelude::*;
use devotee::visual::sprite::Sprite;
use devotee_backend_softbuffer::SoftbufferBackend;

fn main() {
    let init_config = setup::Builder::<Config>::new()
        .with_render_target(Sprite::with_color(0.into()))
        .with_input(Default::default())
        .with_root_constructor(|_| Default::default())
        .with_title("pentacle")
        .with_scale(3);
    let app = app::App::<_, SoftbufferBackend>::with_setup(init_config).unwrap();

    app.run();
}

struct Config;

impl config::Config for Config {
    type Root = Pentacle;
    type Converter = Converter;
    type Input = KeyMouse;
    type RenderTarget = Sprite<SummationPalette, 128, 128>;

    fn converter() -> Self::Converter {
        Converter
    }

    fn background_color() -> SummationPalette {
        0.into()
    }
}

#[derive(Default)]
struct Pentacle {
    rotation: f64,
    counter: f64,
}

impl Root<Config> for Pentacle {
    fn update(&mut self, update: &mut Context<Config>) {
        if update.input().keys().just_pressed(VirtualKeyCode::Escape) {
            update.shutdown();
        }
        let delta = update.delta().as_secs_f64();
        if update.input().keys().is_pressed(VirtualKeyCode::Z) {
            self.rotation += delta;
        }
        self.counter += delta;
    }

    fn render(&self, render: &mut Sprite<SummationPalette, 128, 128>) {
        let mut render = render.painter();

        render.clear(0.into());

        let radius = 48.0 + 8.0 * self.rotation.sin();
        let center = Vector::new(64, 64);

        render.circle((64, 64), radius as i32, draw(2.into()));
        render.circle_f((64, 64), 32, draw(2.into()));

        let radius = radius + 8.0;

        let vertices: Vec<_> = (0..5)
            .map(|i| {
                let a = 0.25 * self.rotation + 2.0 / 5.0 * i as f64 * consts::TAU;

                center
                    + (
                        (radius * a.cos()).round() as i32,
                        (radius * a.sin()).round() as i32,
                    )
            })
            .collect();

        render.polygon(&vertices, draw(2.into()));
        if self.counter.round() as i32 % 2 == 1 {
            render.polygon_f(&vertices, draw(3.into()));
        }

        render.rect((0, 0), (128, 128), draw(2.into()));
    }
}

#[derive(Copy, Clone, PartialEq)]
struct SummationPalette {
    value: u8,
}

impl From<u8> for SummationPalette {
    #[inline]
    fn from(value: u8) -> Self {
        Self { value }
    }
}

impl Color for SummationPalette {
    fn mix(self, other: Self) -> Self {
        Self {
            value: self.value.saturating_add(other.value),
        }
    }
}

struct Converter;

impl color::Converter for Converter {
    type Palette = SummationPalette;
    #[inline]
    fn convert(&self, color: &Self::Palette) -> u32 {
        let brightness = color.value.saturating_mul(64);

        if brightness > 0x80 {
            (brightness as u32) << 16
        } else {
            (brightness as u32) << 16 | (brightness as u32) << 8 | brightness as u32
        }
    }
}
