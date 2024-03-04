use std::time::Duration;

use devotee::app::root::Root;
use devotee::app::{App, AppContext};
use devotee::input::winit_input::Mouse;
use devotee::util::vector::Vector;
use devotee::visual::canvas::Canvas;
use devotee::visual::{paint, Image, Paint, PaintTarget};
use devotee_backend::Converter;
use devotee_backend_softbuffer::{SoftBackend, SoftMiddleware};
use winit::event::MouseButton;

fn main() {
    let backend = SoftBackend::try_new("minimal").unwrap();
    backend
        .run::<App<_>, _, u32, _, Mouse>(
            App::new(Minimal::default()),
            SoftMiddleware::new(Canvas::with_resolution(0x00000000, 128, 128), Mouse::new())
                .with_background_color(0xff804000),
            Duration::from_secs_f32(1.0 / 60.0),
        )
        .unwrap();
}

#[derive(Default)]
struct Minimal {
    cursor: Vector<i32>,
}

impl Root for Minimal {
    type Input = Mouse;
    type Converter = FallThroughConverter;
    type RenderSurface = Canvas<u32>;

    fn update(&mut self, update: AppContext<Mouse>) {
        let input = update.input();
        if input.is_pressed(MouseButton::Left) {
            self.cursor = input.position().any();
        }
    }

    fn render(&self, render: &mut Self::RenderSurface) {
        render.clear(0xffff8000);

        let mut painter = render.painter();
        painter.mod_pixel(self.cursor, paint(0xff4040ff));
    }

    fn converter(&self) -> Self::Converter {
        FallThroughConverter
    }

    fn background_color(&self) -> <Self::Converter as devotee_backend::Converter>::Data {
        0x00ffff80
    }
}

struct FallThroughConverter;

impl Converter for FallThroughConverter {
    type Data = u32;

    fn convert(&self, _x: usize, _y: usize, data: Self::Data) -> u32 {
        data
    }
}
