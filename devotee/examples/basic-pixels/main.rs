use devotee::app::root::Root;
use devotee::app::App;
use devotee::input::winit_input::KeyboardMouse;
use devotee::util::vector::Vector;
use devotee::visual::adapter::generic::Adapter;
use devotee::visual::adapter::CopyConverter;
use devotee::visual::{paint, Paint, Painter};
use devotee_backend::middling::MiddlingMiddleware;
use devotee_backend_pixels::{Error, PixelsBackend, PixelsContext, PixelsInit, PixelsSurface};
use winit::keyboard::KeyCode;

fn main() -> Result<(), Error> {
    let basic = Basic::default();
    let app = App::new(basic);
    let middleware = MiddlingMiddleware::new(app, KeyboardMouse::new());
    let mut backend = PixelsBackend::new(middleware);

    backend.run()
}

#[derive(Default)]
pub struct Basic {
    position: Option<Vector<i32>>,
}

impl Root<PixelsInit<'_>, PixelsContext<'_>, KeyboardMouse, PixelsSurface<'_, '_>> for Basic {
    fn init(&mut self, init: &mut PixelsInit<'_>) {
        init.set_render_window_size(320, 240);
        init.window().set_title("Basic demo: press ESC to exit");
    }

    fn update(&mut self, context: &mut PixelsContext<'_>, input: &KeyboardMouse) {
        self.position = input.mouse().position();

        if input.keyboard().just_pressed(KeyCode::Escape) {
            context.shutdown();
        }
    }

    fn render(&mut self, surface: &mut PixelsSurface<'_, '_>) {
        let converter = CopyConverter::new();
        let mut adapter = Adapter::new(surface, &converter);
        let mut painter = Painter::new(&mut adapter);
        painter.clear([0x40, 0x40, 0x40, 0xff]);
        if let Some(position) = self.position {
            painter.line((0, 0).into(), position, paint([0xff, 0xff, 0xff, 0xff]));
        }
    }
}
