use devotee::app;
use devotee::app::context::Context;
use devotee::app::input::key_mouse::{KeyMouse, VirtualKeyCode};
use devotee::app::root::{ExitPermission, Root};
use devotee::app::setup;
use devotee::util::vector::Vector;
use devotee::visual::canvas::Canvas;
use devotee::visual::prelude::*;
use devotee_backend_softbuffer::SoftbufferBackend;

const BOX_BOUNDARIES: (i32, i32) = (16, 128 - 16);
const INTERNAL_RADIUS: i32 = 8;

fn main() {
    let init_config = setup::Builder::new()
        .with_render_target(Canvas::with_resolution(Default::default(), 128, 128))
        .with_input(Default::default())
        .with_root_constructor(|_| Default::default())
        .with_title("minimal")
        .with_scale(2);
    let app = app::App::<Minimal, SoftbufferBackend>::with_setup(init_config).unwrap();

    app.run();
}

#[derive(Default)]
struct Minimal {
    position: Vector<i32>,
    exit_was_requested: bool,
}

impl Root for Minimal {
    type Converter = Converter;
    type Input = KeyMouse;
    type RenderTarget = Canvas<Color>;

    fn update(&mut self, update: &mut Context<KeyMouse>) {
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
        let mut render = render.painter();

        if self.exit_was_requested {
            render.clear(Color([0x80, 0x00, 0x00]));
        } else {
            render.clear(Color([0x00, 0x00, 0x80]));
        }

        render.rect(
            (BOX_BOUNDARIES.0, BOX_BOUNDARIES.0),
            (BOX_BOUNDARIES.1, BOX_BOUNDARIES.1),
            paint(Color([0xff, 0xff, 0xff])),
        );
        render.circle_f(
            self.position,
            INTERNAL_RADIUS,
            paint(Color([0x80, 0x80, 0x80])),
        );
        render.line(
            (64, 64).into(),
            self.position,
            paint(Color([0xff, 0xff, 0xff])),
        );
        render.line(
            self.position,
            (64, 64).into(),
            paint(Color([0x00, 0xff, 0x00])),
        );
        render.mod_pixel((64, 64), paint(Color([0xff, 0x00, 0x00])));
        render.mod_pixel(self.position, paint(Color([0xff, 0x00, 0x00])));
    }

    fn handle_exit_request(&mut self) -> ExitPermission {
        if self.exit_was_requested {
            ExitPermission::Allow
        } else {
            self.exit_was_requested = true;
            ExitPermission::Forbid
        }
    }

    fn converter(&self) -> &Converter {
        &Converter
    }

    fn background_color(&self) -> Color {
        if self.exit_was_requested {
            Color([0x80, 0x00, 0x00])
        } else {
            Color([0x00, 0x00, 0x80])
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Color([u8; 3]);

struct Converter;

impl devotee_backend::Converter for Converter {
    type Palette = Color;
    fn convert(&self, color: &Self::Palette) -> u32 {
        ((color.0[0] as u32) << 16) | ((color.0[1] as u32) << 8) | (color.0[2] as u32)
    }
}
