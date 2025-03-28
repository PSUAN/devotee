use std::env;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use devotee::app::root::Root;
use devotee::app::App;
use devotee::input::winit_input::KeyboardMouse;
use devotee::util::vector::Vector;
use devotee::visual::adapter::generic::Adapter;
use devotee::visual::adapter::Converter;
use devotee::visual::image::ImageMut;
use devotee::visual::{paint, Paint, Painter};
use devotee_backend::middling::{
    MiddlingMiddleware, Surface, TexelDesignatorMut, TexelDesignatorRef,
};
use devotee_backend_pixels::{PixelsBackend, PixelsContext, PixelsInit, PixelsSurface};
use devotee_backend_softbuffer::{SoftBackend, SoftContext, SoftInit, SoftSurface};

fn main() {
    let style = env::var("ADAPT_STYLE").ok();

    match style {
        Some(pixels) if pixels.to_lowercase().trim() == "pixels" => {
            let internal = Internal::default();
            let adapting = Adapting { internal };
            let app = App::new(adapting);
            let middleware = MiddlingMiddleware::new(app, KeyboardMouse::new());
            let mut backend = PixelsBackend::new(middleware);
            backend.run().unwrap();
        }
        Some(soft) if soft.to_lowercase().trim() == "soft" => {
            let internal = Internal::default();
            let adapting = Adapting { internal };
            let app = App::new(adapting);
            let middleware = MiddlingMiddleware::new(app, KeyboardMouse::new());
            let mut backend = SoftBackend::new(middleware);

            backend.run().unwrap();
        }
        _ => {
            println!("Set an environment variable `ADAPT_STYLE` to either `pixels` or `soft` value")
        }
    }
}

#[derive(Default)]
struct Internal {
    position: Option<Vector<f32>>,
}

impl Internal {
    fn update(&mut self, duration: Duration, input: &KeyboardMouse) {
        if let Some(mouse) = input.mouse().position() {
            self.position = Some(mouse.map(|v| v as _));
        } else if let Some(position) = &mut self.position {
            *position.y_mut() += 32.0 * duration.as_secs_f32();
        }
    }

    fn render<Cfg>(&mut self, render: &mut RenderImage<Cfg::Surface<'_, '_>, Cfg::Converter>)
    where
        Cfg: Config,
    {
        if let Some(position) = self.position {
            let mut painter = Painter::<'_, _, f32>::new(render);
            painter.line((0.0, 0.0).into(), position, paint(Color::Light));
        }
    }
}

type RenderImage<'a, 'b, S, C> = Adapter<'a, 'b, S, C>;

trait Config {
    type Surface<'a, 'b: 'a>: Surface<Texel = <Self::Converter as Converter>::Texel>
        + for<'t> TexelDesignatorRef<
            't,
            TexelRef: Deref<Target = <Self::Converter as Converter>::Texel>,
        > + for<'t> TexelDesignatorMut<
            't,
            TexelMut: DerefMut<Target = <Self::Converter as Converter>::Texel>,
        >;
    type Converter: Converter<Pixel = Color, Texel: Clone>;
}

struct Adapting {
    internal: Internal,
}

impl Root<PixelsInit<'_>, PixelsContext<'_>, KeyboardMouse, PixelsSurface<'_, '_>> for Adapting {
    fn init(&mut self, init: &mut PixelsInit<'_>) {
        init.set_render_window_size(320, 240);
        init.set_title("Adapting demo: Pixels version");
    }

    fn update(&mut self, context: &mut PixelsContext<'_>, input: &KeyboardMouse) {
        self.internal.update(context.delta(), input);
    }

    fn render(&mut self, surface: &mut PixelsSurface<'_, '_>) {
        let mut adapter = Adapter::new(surface, &PixelsConverter);
        adapter.clear(Color::Dark);
        self.internal.render::<PixelsConfig>(&mut adapter);
    }
}

struct PixelsConfig;

impl Config for PixelsConfig {
    type Surface<'a, 'b: 'a> = PixelsSurface<'a, 'b>;

    type Converter = PixelsConverter;
}

struct PixelsConverter;

impl Converter for PixelsConverter {
    type Pixel = Color;
    type Texel = [u8; 3];

    fn forward(&self, pixel: &Self::Pixel) -> Self::Texel {
        match pixel {
            Color::Dark => [0x40, 0x20, 0x20],
            Color::Light => [0xe0; 3],
        }
    }

    fn inverse(&self, texel: &Self::Texel) -> Self::Pixel {
        if texel[0] > 0x80 {
            Color::Light
        } else {
            Color::Dark
        }
    }
}

impl Root<SoftInit<'_>, SoftContext<'_>, KeyboardMouse, SoftSurface<'_>> for Adapting {
    fn init(&mut self, init: &mut SoftInit<'_>) {
        init.set_render_window_size(320, 240);
        init.set_title("Adapting demo: SoftBuffer version");
    }

    fn update(&mut self, context: &mut SoftContext<'_>, input: &KeyboardMouse) {
        self.internal.update(context.delta(), input);
    }

    fn render(&mut self, surface: &mut SoftSurface<'_>) {
        let mut adapter = Adapter::new(surface, &SoftConverter);
        adapter.clear(Color::Dark);
        self.internal.render::<SoftConfig>(&mut adapter);
    }
}

struct SoftConfig;

impl Config for SoftConfig {
    type Surface<'a, 'b: 'a> = SoftSurface<'a>;

    type Converter = SoftConverter;
}

struct SoftConverter;

impl Converter for SoftConverter {
    type Pixel = Color;
    type Texel = u32;

    fn forward(&self, pixel: &Self::Pixel) -> Self::Texel {
        match pixel {
            Color::Dark => 0x202040,
            Color::Light => 0xe0e0e0,
        }
    }

    fn inverse(&self, _: &Self::Texel) -> Self::Pixel {
        Color::Dark
    }
}

#[derive(Clone, Copy, Debug)]
enum Color {
    Dark,
    Light,
}
