use devotee::graphics::ugly_graphics::surface_adapter::SurfaceAdapter;
use devotee::input::winit_input::KeyboardMouse;
use devotee::util::vector::Vector;
use devotee_backend::Middleware;
use devotee_backend::middling::InputHandler;
use devotee_backend_pixels::{
    Error, PixelsBackend, PixelsContext, PixelsEvent, PixelsEventContext, PixelsInit, PixelsSurface,
};
use ugly_graphics::image::ImageMut;
use ugly_graphics::operation::pixel::Pixel;
use ugly_graphics::operation::scanline::line::Line;
use ugly_graphics::painter::Painter;
use ugly_graphics::strategy;
use winit::keyboard::KeyCode;

fn main() -> Result<(), Error> {
    let basic = Basic::default();
    let mut backend = PixelsBackend::new(basic);

    backend.run()
}

#[derive(Default)]
pub struct Basic {
    input: KeyboardMouse,
    position: Option<Vector<i32>>,
}

impl
    Middleware<
        PixelsInit<'_>,
        PixelsContext<'_>,
        PixelsSurface<'_, '_>,
        PixelsEvent,
        PixelsEventContext<'_, '_, '_>,
    > for Basic
{
    fn on_init(&mut self, init: &mut PixelsInit<'_>) {
        init.set_render_window_size(160, 120);
        init.window().set_title("Basic demo: press ESC to exit");
    }

    fn on_update(&mut self, context: &mut PixelsContext<'_>) {
        self.position = self.input.mouse().position();

        if self.input.keyboard().just_pressed(KeyCode::Escape) {
            context.shutdown();
        }

        InputHandler::<_, PixelsEventContext>::update(&mut self.input);
    }

    fn on_render(&mut self, surface: &mut PixelsSurface<'_, '_>) {
        let mut adapter = SurfaceAdapter::new(surface);

        adapter.write(&strategy::overwrite([0x20, 0x40, 0x60, 0xff]));

        let mut painter = Painter::new(&mut adapter);

        if let Some(position) = self.position {
            painter.draw(Line::new(
                (0, 0),
                position.split(),
                strategy::overwrite([0xff, 0xff, 0xff, 0xff]),
            ));
            painter.draw(Line::new(
                (159, 119),
                position.split(),
                strategy::overwrite([0x00, 0xff, 0x00, 0xff]),
            ));
            painter.draw(Line::new(
                (80, 60),
                position.split(),
                strategy::overwrite([0x00, 0x00, 0x00, 0xff]),
            ));
            painter.draw(Pixel::new(
                (80, 60),
                strategy::overwrite([0xff, 0x00, 0x00, 0xff]),
            ));
        }
    }

    fn on_event(
        &mut self,
        event: PixelsEvent,
        event_context: &mut PixelsEventContext<'_, '_, '_>,
    ) -> Option<PixelsEvent> {
        if let PixelsEvent::Window(window_event) = event {
            self.input
                .handle_event(window_event, event_context)
                .map(PixelsEvent::Window)
        } else {
            Some(event)
        }
    }
}
