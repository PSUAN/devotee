use pixels::{wgpu, Error, Pixels, PixelsBuilder, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
use winit::window::{Fullscreen, Window as WinitWindow, WindowBuilder};

use super::{Config, Setup};
use crate::util::vector::Vector;
use crate::visual::color::Converter;
use crate::visual::Draw;

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
        Cfg::RenderTarget: Draw,
        Cfg::Converter: Converter<Palette = <Cfg::RenderTarget as Draw>::Pixel>,
    {
        let resolution = Vector::new(
            setup.render_target.width() as u32,
            setup.render_target.height() as u32,
        );
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
        window.set_cursor_visible(false);
        #[cfg(target_arch = "wasm32")]
        {
            let document = web_sys::window()?.document()?;
            if let Some(canvas_holder) =
                document.get_element_by_id(setup.element_id.unwrap_or("devoteeCanvasHolder"))
            {
                let _ = canvas_holder.append_child(&web_sys::Element::from(window.canvas()));
            } else {
                let _ = document
                    .body()?
                    .append_child(&web_sys::Element::from(window.canvas()));
            }
        }

        let pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            let background_color = setup.background_color;
            let builder = PixelsBuilder::new(resolution.x(), resolution.y(), surface_texture)
                .clear_color(wgpu::Color {
                    r: background_color[0] as f64 / 255.0,
                    g: background_color[1] as f64 / 255.0,
                    b: background_color[2] as f64 / 255.0,
                    a: 1.0,
                });
            #[cfg(target_arch = "wasm32")]
            {
                pollster::block_on(builder.build_async()).ok()?
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let builder = builder
                    .surface_texture_format(wgpu::TextureFormat::Bgra8Unorm)
                    .render_texture_format(wgpu::TextureFormat::Bgra8Unorm)
                    .texture_format(wgpu::TextureFormat::Rgba8Unorm);
                builder.build().ok()?
            }
        };

        Some(Window {
            window,
            pixels,
            resolution,
        })
    }

    pub(super) fn pixels(&self) -> &Pixels {
        &self.pixels
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

    /// Get window pixel resolution.
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

    pub(super) fn apply(&mut self, commands: &mut Vec<WindowCommand>) {
        for command in commands.drain(..) {
            command(self)
        }
    }
}
