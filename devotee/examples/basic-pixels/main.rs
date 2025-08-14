use devotee::input::winit_input::KeyboardMouse;
use devotee::util::vector::Vector;
use devotee::visual::adapter::CopyConverter;
use devotee::visual::adapter::generic::Adapter;
use devotee::visual::{Paint, Painter};
use devotee_backend::Middleware;
use devotee_backend::middling::InputHandler;
use devotee_backend_pixels::{
    Error, PixelsBackend, PixelsContext, PixelsEvent, PixelsEventContext, PixelsEventControl,
    PixelsInit, PixelsSurface,
};
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
        PixelsEventContext<'_, '_>,
        PixelsEventControl<'_>,
    > for Basic
{
    fn on_init(&mut self, init: &mut PixelsInit<'_>) {
        init.set_render_window_size(320, 240);
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
        let converter = CopyConverter::new();
        let mut adapter = Adapter::new(surface, &converter);

        let mut painter = Painter::new(&mut adapter);
        painter.clear([0x20, 0x40, 0x60, 0xff]);

        if let Some(position) = self.position {
            painter.line((0, 0).into(), position, &[0xff, 0xff, 0xff, 0xff]);
            painter.line(
                (319, 239).into(),
                position,
                &mut |_, [r, g, b, a]: [u8; 4]| [b, r, g, a],
            );
        }
    }

    fn on_event(
        &mut self,
        event: PixelsEvent,
        event_context: &PixelsEventContext<'_, '_>,
        _: &mut PixelsEventControl<'_>,
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
