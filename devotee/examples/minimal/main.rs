use std::time::Duration;

use devotee::app::root::Root;
use devotee::app::App;
use devotee::input::winit_input::NoInput;
use devotee::visual::canvas::Canvas;
use devotee_backend::Converter;
use devotee_backend_softbuffer::{Error, SoftBackend, SoftContext, SoftInit, SoftMiddleware};

fn main() -> Result<(), Error> {
    let backend = SoftBackend::try_new("minimal")?;
    backend.run(
        App::new(Minimal),
        SoftMiddleware::new(Canvas::with_resolution(false, 128, 128), NoInput),
        Duration::from_secs_f32(1.0 / 60.0),
    )
}

struct Minimal;

impl Root<SoftInit<'_>, SoftContext<'_, NoInput>> for Minimal {
    type Converter = BlackWhiteConverter;
    type RenderSurface = Canvas<bool>;

    fn init(&mut self, _: &mut SoftInit) {}

    fn update(&mut self, _: &mut SoftContext<NoInput>) {}

    fn render(&self, _: &mut Self::RenderSurface) {}

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
