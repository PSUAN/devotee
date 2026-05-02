use devotee::graphics::ugly_graphics::{RowIterator, RowIteratorMapper};
use devotee::input::winit_input::KeyboardMouse;
use devotee::util::vector::Vector;
use devotee_backend::Middleware;
use devotee_backend::middling::{Fill, InputHandler};
use devotee_backend_pixels::{
    Error, PixelsBackend, PixelsContext, PixelsEvent, PixelsEventContext, PixelsInit, PixelsSurface,
};
use ugly_graphics::image::sprite::Sprite;
use ugly_graphics::image::{Dimensions, ImageMut};
use ugly_graphics::operation::scanline::line::Line;
use ugly_graphics::painter::Painter;
use ugly_graphics::strategy;
use winit::keyboard::KeyCode;

fn main() -> Result<(), Error> {
    let basic = Basic::default();

    let mut backend = PixelsBackend::new(basic);

    backend.run()
}

pub struct Basic {
    sprite: Sprite<bool, 128, 64>,
    input: KeyboardMouse,
    position: Option<Vector<i32>>,
}

impl Default for Basic {
    fn default() -> Self {
        let sprite = Sprite::from_copies(false);
        Self {
            sprite,
            input: Default::default(),
            position: Default::default(),
        }
    }
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
    fn on_init(&mut self, init: &mut PixelsInit) {
        let (width, height) = self.sprite.dimensions();
        init.set_render_window_size(width, height);
    }

    fn on_update(&mut self, context: &mut PixelsContext) {
        self.position = self.input.mouse().position();

        if self.input.keyboard().just_pressed(KeyCode::Escape) {
            context.shutdown();
        }

        InputHandler::<_, PixelsEventContext>::update(&mut self.input);
    }

    fn on_render(&mut self, surface: &mut PixelsSurface) {
        self.sprite.set(false);
        let mut painter = Painter::new(&mut self.sprite);
        if let Some(position) = self.position {
            painter.draw(Line::new(
                (0, 0),
                position.split(),
                strategy::overwrite(true),
            ));
        }

        surface.fill_from(
            RowIteratorMapper::new(&self.sprite, |value| {
                if value {
                    [0x80, 0xff, 0xff, 0xff]
                } else {
                    [0x10, 0x20, 0x20, 0xff]
                }
            })
            .row_iterator(),
        );
    }

    fn on_event(
        &mut self,
        event: PixelsEvent,
        event_context: &mut PixelsEventContext,
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
