use super::{Config, Setup};
use crate::math::vector::Vector;
use crate::visual::color::Converter;
use pixels::{wgpu, Error, Pixels, PixelsBuilder, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
use winit::window::{Fullscreen, Window as WinitWindow, WindowBuilder};

pub(super) type WindowCommand = Box<dyn FnOnce(&mut Window)>;

/// The application window.
pub struct Window {
    window: WinitWindow,
    pixels: Pixels,
    resolution: Vector<u32>,
}

impl Window {
    pub(super) fn with_setup<Cfg>(event_loop: &EventLoop<()>, setup: &Setup<Cfg>) -> Option<Self>
    where
        Cfg: Config,
        Cfg::Converter: Converter<Palette = Cfg::Palette>,
    {
        let resolution = Vector::new(setup.resolution.x() as u32, setup.resolution.y() as u32);
        if resolution.x() == 0 || resolution.y() == 0 {
            return None;
        }
        if setup.scale == 0 {
            return None;
        }
        let window_size = resolution * setup.scale;

        let window = {
            let builder = WindowBuilder::new()
                .with_inner_size(PhysicalSize::new(window_size.x(), window_size.y()))
                .with_fullscreen(if setup.fullscreen {
                    Some(Fullscreen::Borderless(None))
                } else {
                    None
                })
                .with_title(&setup.title);
            builder.build(event_loop).ok()?
        };
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                });
        }

        let pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            let clear_color = Cfg::window_background_color();
            let builder = PixelsBuilder::new(resolution.x(), resolution.y(), surface_texture)
                .clear_color(wgpu::Color {
                    r: clear_color[0] as f64 / 255.0,
                    g: clear_color[1] as f64 / 255.0,
                    b: clear_color[2] as f64 / 255.0,
                    a: 1.0,
                });
            #[cfg(target_arch = "wasm32")]
            {
                pollster::block_on(builder.build_async()).ok()?
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                builder.build().ok()?
            }
        };

        Some(Window {
            window,
            pixels,
            resolution,
        })
    }

    pub(super) fn pixels_mut(&mut self) -> &mut Pixels {
        &mut self.pixels
    }

    pub(super) fn render(&mut self) -> Result<(), Error> {
        self.pixels.render()
    }

    pub(super) fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// Get window resolution
    pub fn resolution(&self) -> Vector<u32> {
        self.resolution
    }

    /// Set window to fullscreen mode.
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        if fullscreen {
            self.window
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        } else {
            self.window.set_fullscreen(None);
        }
    }

    /// Check if the window is in fullscreen mode.
    pub fn is_fullscreen(&self) -> bool {
        self.window.fullscreen().is_some()
    }

    pub(super) fn apply(&mut self, commands: Vec<WindowCommand>) {
        for command in commands {
            command(self)
        }
    }
}
