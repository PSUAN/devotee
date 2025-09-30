#![deny(missing_docs)]

//! [Pixels](https://crates.io/crates/pixels)-based backend for the devotee project.

use std::sync::Arc;
use std::time::{Duration, Instant};

use devotee_backend::Middleware;
use devotee_backend::middling::{EventContext, Fill, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::error::{EventLoopError, OsError};
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

pub use winit;

/// A [Pixels](https://crates.io/crates/pixels)-based backend implementation for the devotee project.
pub struct PixelsBackend<'w, M> {
    middleware: M,
    internal: Option<Internal<'w>>,
    settings: Settings,
    last: Instant,
}

impl<M> PixelsBackend<'_, M> {
    /// Create new backend instance.
    pub fn new(middleware: M) -> Self {
        let internal = None;
        let last = Instant::now();
        let settings = Settings {
            render_window_size: PhysicalSize::new(32, 32),
            updates_per_second: 60.0,
        };
        Self {
            middleware,
            internal,
            settings,
            last,
        }
    }
}

impl<'w, M> PixelsBackend<'w, M>
where
    for<'init, 'context, 'surface, 'event_context, 'event_control> M: Middleware<
            PixelsInit<'init>,
            PixelsContext<'context>,
            PixelsSurface<'surface, 'w>,
            PixelsEvent,
            PixelsEventContext<'event_context, 'w>,
            PixelsEventControl<'event_control>,
        >,
{
    /// Run this backend to completion.
    pub fn run(&mut self) -> Result<(), Error> {
        let event_loop = EventLoop::new()?;

        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_secs_f32(1.0 / 60.0),
        ));
        event_loop.run_app(self)?;

        Ok(())
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Error> {
        let window = Arc::new(event_loop.create_window(WindowAttributes::default())?);

        let mut init = PixelsInit {
            window: &window,
            settings: &mut self.settings,
        };

        window.set_visible(true);
        self.middleware.on_init(&mut init);
        window.set_min_inner_size(Some(self.settings.render_window_size));

        let surface_size = window.inner_size();
        let surface_texture =
            pixels::SurfaceTexture::new(surface_size.width, surface_size.height, window.clone());

        let pixels = pixels::PixelsBuilder::new(
            self.settings.render_window_size.width,
            self.settings.render_window_size.height,
            surface_texture,
        )
        .build()?;

        let mut internal = Internal {
            window,
            pixels,
            surface_size,
        };
        let _ = internal.on_resize(surface_size);

        self.internal = Some(internal);

        Ok(())
    }

    fn handle_update(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if let Some(internal) = &self.internal {
            let delta = now - self.last;

            let mut control = PixelsContext {
                shutdown: false,
                window: &internal.window,
                delta,
            };
            self.middleware.on_update(&mut control);

            if control.shutdown {
                event_loop.exit();
            }
        }
        self.last = now;
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            now + Duration::from_secs_f32(1.0 / self.settings.updates_per_second),
        ));
    }

    fn handle_window_event(
        settings: &Settings,
        event_loop: &ActiveEventLoop,
        middleware: &mut M,
        internal: &mut Internal<'w>,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(physical_size) => {
                internal.on_resize(physical_size);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Destroyed => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let mut surface = PixelsSurface {
                    pixels: &mut internal.pixels,
                    dimensions: settings.render_window_size,
                };
                middleware.on_render(&mut surface);
                let _ = internal.pixels.render();
                internal.window.request_redraw();
            }
            _ => {}
        }
    }

    fn handle_event(
        settings: &Settings,
        event_loop: &ActiveEventLoop,
        middleware: &mut M,
        internal: &mut Internal<'w>,
        event: PixelsEvent,
    ) {
        if let PixelsEvent::Window(event) = event {
            Self::handle_window_event(settings, event_loop, middleware, internal, event)
        }
    }
}

impl<'w, M> ApplicationHandler for PixelsBackend<'w, M>
where
    for<'init, 'control, 'surface, 'event_context, 'event_control> M: Middleware<
            PixelsInit<'init>,
            PixelsContext<'control>,
            PixelsSurface<'surface, 'w>,
            PixelsEvent,
            PixelsEventContext<'event_context, 'w>,
            PixelsEventControl<'event_control>,
        >,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.internal.is_none() && self.init(event_loop).is_err() {
            event_loop.exit();
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            self.handle_update(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(internal) = &mut self.internal {
            let surface = PixelsSurface {
                pixels: &mut internal.pixels,
                dimensions: self.settings.render_window_size,
            };
            let context = PixelsEventContext { surface };
            let mut control = PixelsEventControl { event_loop };

            if let Some(event) =
                self.middleware
                    .on_event(PixelsEvent::Window(event), &context, &mut control)
            {
                Self::handle_event(
                    &self.settings,
                    event_loop,
                    &mut self.middleware,
                    internal,
                    event,
                );
            }
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(internal) = &mut self.internal {
            let surface = PixelsSurface {
                pixels: &mut internal.pixels,
                dimensions: self.settings.render_window_size,
            };
            let context = PixelsEventContext { surface };
            let mut control = PixelsEventControl { event_loop };
            let _ = self.middleware.on_event(
                PixelsEvent::Device(device_id, event),
                &context,
                &mut control,
            );
        }
    }
}

struct Settings {
    render_window_size: PhysicalSize<u32>,
    updates_per_second: f32,
}

impl Settings {
    /// # Panics
    ///
    /// Panics if `updates_per_second` is less or equal to `0`.
    fn set_updates_per_second(&mut self, updates_per_second: f32) {
        assert!(
            updates_per_second > 0.0,
            "Update rate has to be greater than 0"
        );
        self.updates_per_second = updates_per_second;
    }
}

struct Internal<'w> {
    window: Arc<Window>,
    pixels: pixels::Pixels<'w>,
    surface_size: PhysicalSize<u32>,
}

impl Internal<'_> {
    fn on_resize(&mut self, size: PhysicalSize<u32>) -> Option<()> {
        self.pixels.resize_surface(size.width, size.height).ok()?;

        self.surface_size = size;

        Some(())
    }
}

/// An initialization argument passed to the application.
pub struct PixelsInit<'a> {
    window: &'a Window,
    settings: &'a mut Settings,
}

impl PixelsInit<'_> {
    /// Get reference to the underlying `winit` `Window` reference.
    pub fn window(&self) -> &Window {
        self.window
    }

    /// Set the internal render window size.
    pub fn set_render_window_size(&mut self, width: u32, height: u32) {
        let size = PhysicalSize::new(width, height);
        self.settings.render_window_size = size;
        let _ = self.window.request_inner_size(size);
    }

    /// Set update framerate in updates per second count.
    ///
    /// # Panics
    ///
    /// Panics if `updates_per_second` is less or equal to `0`.
    pub fn set_updates_per_second(&mut self, updates_per_second: f32) {
        self.settings.set_updates_per_second(updates_per_second);
    }
}

/// An update argument passed to the application.
pub struct PixelsContext<'a> {
    shutdown: bool,
    window: &'a Window,
    delta: Duration,
}

impl PixelsContext<'_> {
    /// Get reference to the underlying `winit` `Window` reference.
    pub fn window(&self) -> &Window {
        self.window
    }

    /// Tell the backend to shut itself down.
    pub fn shutdown(&mut self) {
        self.shutdown = true;
    }

    /// Get delta time estimation.
    pub fn delta(&self) -> Duration {
        self.delta
    }
}

/// Pixels-based backend event.
pub enum PixelsEvent {
    /// Winit window event.
    Window(WindowEvent),
    /// Winit device event.
    Device(DeviceId, DeviceEvent),
}

/// A context passed to the event handler.
pub struct PixelsEventContext<'s, 'w> {
    surface: PixelsSurface<'s, 'w>,
}

impl EventContext<PhysicalPosition<f64>> for PixelsEventContext<'_, '_> {
    type SurfaceSpace = Option<PhysicalPosition<u32>>;

    fn estimate_surface_space(&self, event_space: PhysicalPosition<f64>) -> Self::SurfaceSpace {
        let PhysicalPosition { x, y } = event_space;
        let (x, y) = (x as f32, y as f32);
        self.surface
            .pixels
            .window_pos_to_pixel((x, y))
            .ok()
            .map(|(x, y)| PhysicalPosition::new(x as _, y as _))
    }
}

/// Render surface implementation.
pub struct PixelsSurface<'s, 'w> {
    pixels: &'s mut pixels::Pixels<'w>,
    dimensions: PhysicalSize<u32>,
}

impl Surface for PixelsSurface<'_, '_> {
    type Texel = [u8; 4];

    fn texel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.dimensions.width || y >= self.dimensions.height {
            None
        } else {
            let buffer = self.pixels.frame();
            let offset = (4 * (x + y * self.dimensions.width)) as usize;
            let slice = &buffer[offset..(offset + 4)];
            slice.try_into().ok()
        }
    }

    fn set_texel(&mut self, x: u32, y: u32, value: [u8; 4]) {
        if x < self.dimensions.width && y < self.dimensions.height {
            let buffer = self.pixels.frame_mut();
            let offset = (4 * (x + y * self.dimensions.width)) as usize;
            let slice = &mut buffer[offset..(offset + 4)];
            slice.copy_from_slice(&value);
        }
    }

    unsafe fn texel_unchecked(&self, x: u32, y: u32) -> [u8; 4] {
        let buffer = self.pixels.frame();
        let offset = (4 * (x + y * self.dimensions.width)) as usize;
        let slice = &buffer[offset..(offset + 4)];
        slice.try_into().unwrap()
    }

    unsafe fn set_texel_unchecked(&mut self, x: u32, y: u32, value: [u8; 4]) {
        let buffer = self.pixels.frame_mut();
        let offset = (4 * (x + y * self.dimensions.width)) as usize;
        let slice = &mut buffer[offset..(offset + 4)];
        slice.copy_from_slice(&value);
    }

    fn clear(&mut self, value: Self::Texel) {
        let frame = self.pixels.frame_mut();
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&value);
        }
    }

    fn width(&self) -> u32 {
        self.dimensions.width
    }

    fn height(&self) -> u32 {
        self.dimensions.height
    }
}

impl Fill for PixelsSurface<'_, '_> {
    fn fill_from(&mut self, data: &[Self::Texel]) {
        let frame = self.pixels.frame_mut();
        for (texel, data) in frame.chunks_exact_mut(4).zip(data) {
            texel.copy_from_slice(data);
        }
    }
}

/// Event control structure.
pub struct PixelsEventControl<'a> {
    event_loop: &'a ActiveEventLoop,
}

impl PixelsEventControl<'_> {
    /// Tell the backend to shut itself down.
    pub fn shutdown(&self) {
        self.event_loop.exit();
    }
}

/// An error generalization.
#[derive(Debug)]
pub enum Error {
    /// Winit event loop error.
    WinitEventLoopError(EventLoopError),
    /// Os error.
    OsError(OsError),
    /// Pixels error.
    PixelsError(pixels::Error),
}

impl From<EventLoopError> for Error {
    fn from(value: EventLoopError) -> Self {
        Self::WinitEventLoopError(value)
    }
}

impl From<OsError> for Error {
    fn from(value: OsError) -> Self {
        Self::OsError(value)
    }
}

impl From<pixels::Error> for Error {
    fn from(value: pixels::Error) -> Self {
        Self::PixelsError(value)
    }
}
