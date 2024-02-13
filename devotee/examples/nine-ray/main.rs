use std::f32::consts;

use devotee::app;
use devotee::app::context::Context;
use devotee::app::input::key_mouse::{KeyMouse, VirtualKeyCode};
use devotee::app::root::Root;
use devotee::app::setup;
use devotee::util::vector::Vector;
use devotee::visual::prelude::*;
use devotee::visual::sprite::Sprite;
use devotee_backend_softbuffer::SoftbufferBackend;

fn main() {
    let init_config = setup::Builder::new()
        .with_render_target(Sprite::with_color(0.into()))
        .with_input(Default::default())
        .with_root_constructor(|_| Default::default())
        .with_title("nine-ray")
        .with_scale(3);
    let app = app::App::<NineRay, SoftbufferBackend>::with_setup(init_config).unwrap();

    app.run();
}

#[derive(Default)]
struct NineRay {
    rotation: f32,
    counter: f32,
}

impl Root for NineRay {
    type Converter = Converter;
    type Input = KeyMouse;
    type RenderTarget = Sprite<SummationPalette, 128, 128>;

    fn update(&mut self, update: &mut Context<KeyMouse>) {
        if update.input().keys().just_pressed(VirtualKeyCode::Escape) {
            update.shutdown();
        }
        let delta = update.delta().as_secs_f32();
        if update.input().keys().is_pressed(VirtualKeyCode::Z) {
            self.rotation += delta;
        }
        self.counter += delta;
    }

    fn render(&self, render: &mut Sprite<SummationPalette, 128, 128>) {
        let mut render = render.painter();

        render.clear(0.into());

        let radius = 48.0 + 8.0 * self.rotation.sin();
        let center = Vector::new(64.0, 64.0);

        render.circle_b((64.0, 64.0), radius, draw(2.into()));
        render.circle_f((64.0, 64.0), radius / 4.0, draw(2.into()));

        let radius = radius + 8.0;

        let vertices: Vec<_> = (0..9)
            .map(|i| {
                let a = 0.25 * self.rotation + 4.0 / 9.0 * i as f32 * consts::TAU;

                center + (radius * a.cos(), radius * a.sin())
            })
            .collect();

        render.polygon_b(&vertices, draw(2.into()));
        if self.counter.round() as i32 % 2 == 1 {
            render.polygon_f(&vertices, draw(3.into()));
        }

        render.rect_b((0.0, 0.0), (128.0, 128.0), draw(2.into()));
    }

    fn converter(&self) -> &Converter {
        &Converter
    }

    fn background_color(&self) -> SummationPalette {
        SummationPalette { value: 0 }
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

impl SummationPalette {
    fn mix(self, other: Self) -> Self {
        Self {
            value: self.value.saturating_add(other.value),
        }
    }
}

fn draw(color: SummationPalette) -> impl FnMut(i32, i32, SummationPalette) -> SummationPalette {
    move |_x, _y, original| original.mix(color)
}

struct Converter;

impl devotee_backend::Converter for Converter {
    type Palette = SummationPalette;
    #[inline]
    fn convert(&self, color: &Self::Palette) -> u32 {
        let brightness = color.value.saturating_mul(64);

        if brightness > 0x80 {
            brightness as u32
        } else {
            (brightness as u32) << 16 | (brightness as u32) << 8 | brightness as u32
        }
    }
}
