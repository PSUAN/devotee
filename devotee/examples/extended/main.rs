use std::time::Duration;

use devotee::app::root::Root;
use devotee::app::App;
use devotee::input::winit_input::{KeyCode, Keyboard};
use devotee::util::vector::Vector;
use devotee::visual::canvas::Canvas;
use devotee::visual::{paint, Image, Paint, PaintTarget};
use devotee_backend::{Context, Converter};
use devotee_backend_softbuffer::{Error, SoftBackend, SoftContext, SoftInit, SoftMiddleware};

fn main() -> Result<(), Error> {
    let backend = SoftBackend::try_new("extended")?;
    backend.run(
        App::new(Extended::default()),
        SoftMiddleware::new(Canvas::with_resolution(false, 128, 128), Keyboard::new()),
        Duration::from_secs_f32(1.0 / 60.0),
    )
}

#[derive(Default)]
struct Extended {
    counter: f32,
}

impl Root<SoftInit<'_>, SoftContext<'_, Keyboard>> for Extended {
    type Converter = BlackWhiteConverter;
    type RenderSurface = Canvas<bool>;

    fn init(&mut self, _: &mut SoftInit) {}

    fn update(&mut self, context: &mut SoftContext<Keyboard>) {
        if context.input().just_pressed(KeyCode::Escape) {
            context.shutdown();
        }

        self.counter += context.delta().as_secs_f32();
    }

    fn render(&mut self, surface: &mut Self::RenderSurface) {
        surface.clear(false);
        let center = surface.dimensions().map(|a| a as f32) / 2.0
            + Vector::new(self.counter.cos(), -self.counter.sin()) * 8.0;

        let mut painter = surface.painter();
        let radius = 16.0 + 16.0 * self.counter.sin();

        painter.circle_f(center, radius, paint(true));
        painter.circle_f(center, radius / 2.0, |x, y, _| (x + y) % 2 == 0);
        painter.circle_b(center, radius + 3.0, paint(true));
    }

    fn converter(&self) -> Self::Converter {
        BlackWhiteConverter
    }
}

struct BlackWhiteConverter;

impl Converter for BlackWhiteConverter {
    type Data = bool;

    fn convert(&self, _: usize, _: usize, data: Self::Data) -> u32 {
        if data {
            0xffffffff
        } else {
            0xff000000
        }
    }
}
