#![deny(missing_docs)]

//! [Softbuffer](https://crates.io/crates/softbuffer)-based backend for the devotee project.

use std::rc::Rc;
use std::time::{Duration, Instant};

use devotee_backend::Middleware;
use devotee_backend::middling::EventContext;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::error::{EventLoopError, OsError};
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use surface::ScaleMode;

pub use surface::SoftSurface;
pub use winit;

mod surface;

/// [Softbuffer](https://crates.io/crates/softbuffer)-based backend implementation for the devotee project.
pub struct SoftBackend<M> {
    middleware: M,
    internal: Option<Internal>,
    settings: Settings,
    last: Instant,
}

impl<M> SoftBackend<M> {
    /// Create new backend instance.
    pub fn new(middleware: M) -> Self {
        let internal = None;
        let last = Instant::now();
        let settings = Settings {
            render_window_size: PhysicalSize::new(32, 32),
            border_color: 0,
            scale_mode: ScaleMode::Auto,
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

impl<M> SoftBackend<M>
where
    for<'init, 'context, 'surface, 'control> M: Middleware<
            SoftInit<'init>,
            SoftContext<'context>,
            SoftSurface<'surface>,
            SoftEvent,
            SoftEventContext,
            SoftEventControl<'control>,
        >,
{
    /// Run this backend to completion.
    pub fn run(&mut self) -> Result<(), Error> {
        let event_loop = EventLoop::new()?;

        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_secs_f32(1.0 / self.settings.updates_per_second),
        ));
        event_loop.run_app(self)?;

        Ok(())
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Error> {
        let window = Rc::new(event_loop.create_window(WindowAttributes::default())?);

        let mut init = SoftInit {
            window: &window,
            settings: &mut self.settings,
        };

        window.set_visible(true);
        self.middleware.on_init(&mut init);
        window.set_min_inner_size(Some(self.settings.render_window_size));

        let context = softbuffer::Context::new(Rc::clone(&window))?;
        let surface = softbuffer::Surface::new(&context, Rc::clone(&window))?;
        let surface_size = window.inner_size();

        let mut internal = Internal {
            window,
            surface,
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

            let mut control = SoftContext {
                shutdown: false,
                window: &internal.window,
                delta,
            };
            self.middleware.on_update(&mut control);

            internal.window.request_redraw();

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
        internal: &mut Internal,
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
                if let Ok(buffer) = internal.surface.buffer_mut() {
                    let mut surface = SoftSurface::new(
                        buffer,
                        internal.surface_size,
                        settings.scale_mode,
                        settings.render_window_size,
                    );
                    let _ = surface.clear(settings.border_color);
                    middleware.on_render(&mut surface);
                    let _ = surface.present();
                }
            }
            _ => {}
        }
    }

    fn handle_event(
        settings: &Settings,
        event_loop: &ActiveEventLoop,
        middleware: &mut M,
        internal: &mut Internal,
        event: SoftEvent,
    ) {
        if let SoftEvent::Window(event) = event {
            Self::handle_window_event(settings, event_loop, middleware, internal, event)
        }
    }
}

impl<M> ApplicationHandler for SoftBackend<M>
where
    for<'init, 'context, 'surface, 'control> M: Middleware<
            SoftInit<'init>,
            SoftContext<'context>,
            SoftSurface<'surface>,
            SoftEvent,
            SoftEventContext,
            SoftEventControl<'control>,
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
            let context = SoftEventContext::new(
                internal.surface_size,
                self.settings.scale_mode,
                self.settings.render_window_size,
            );
            let mut control = SoftEventControl { event_loop };

            if let Some(event) =
                self.middleware
                    .on_event(SoftEvent::Window(event), &context, &mut control)
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
            let context = SoftEventContext::new(
                internal.surface_size,
                self.settings.scale_mode,
                self.settings.render_window_size,
            );
            let mut control = SoftEventControl { event_loop };

            let _ = self.middleware.on_event(
                SoftEvent::Device(device_id, event),
                &context,
                &mut control,
            );
        }
    }
}

struct Settings {
    render_window_size: PhysicalSize<u32>,
    border_color: u32,
    scale_mode: ScaleMode,
    updates_per_second: f32,
}

impl Settings {
    fn set_scale(&mut self, scale: u32) {
        if let Ok(scale) = scale.try_into() {
            self.scale_mode = ScaleMode::Fixed(scale);
        } else {
            self.scale_mode = ScaleMode::Auto;
        }
    }

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

struct Internal {
    window: Rc<Window>,
    surface: softbuffer::Surface<Rc<Window>, Rc<Window>>,
    surface_size: PhysicalSize<u32>,
}

impl Internal {
    fn on_resize(&mut self, size: PhysicalSize<u32>) -> Option<()> {
        self.surface
            .resize(size.width.try_into().ok()?, size.height.try_into().ok()?)
            .ok()?;

        self.surface_size = size;

        Some(())
    }
}

/// An initialization argument passed to the application.
pub struct SoftInit<'a> {
    window: &'a Window,
    settings: &'a mut Settings,
}

impl SoftInit<'_> {
    /// Get internal `winit` window reference.
    pub fn window(&self) -> &Window {
        self.window
    }

    /// Set internal render scale.
    /// If case of `0` `scale` value automatic scaling is used.
    pub fn set_scale(&mut self, scale: u32) {
        self.settings.set_scale(scale);
    }

    /// Set update framerate in updates per second count.
    ///
    /// # Panics
    ///
    /// Panics if `updates_per_second` is less or equal to `0`.
    pub fn set_updates_per_second(&mut self, updates_per_second: f32) {
        self.settings.set_updates_per_second(updates_per_second);
    }

    /// Set the internal render window size.
    pub fn set_render_window_size(&mut self, width: u32, height: u32) {
        let size = PhysicalSize::new(width, height);
        self.settings.render_window_size = size;
        let _ = self.window.request_inner_size(size);
    }

    /// Set the color of the border to be rendered.
    pub fn set_border_color(&mut self, color: u32) {
        self.settings.border_color = color;
    }
}

/// An update argument passed to the application.
pub struct SoftContext<'a> {
    shutdown: bool,
    window: &'a Window,
    delta: Duration,
}

impl SoftContext<'_> {
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

/// An event produced by the Softbuffer-backed backend.
pub enum SoftEvent {
    /// Winit window event.
    Window(WindowEvent),
    /// Winit raw device event.
    Device(DeviceId, DeviceEvent),
}

/// A context passed to the event handler.
pub struct SoftEventContext {
    render_window: (PhysicalPosition<u32>, PhysicalSize<u32>),
    scale: u32,
}

impl SoftEventContext {
    fn new(
        surface_size: PhysicalSize<u32>,
        scale: ScaleMode,
        render_window_size: PhysicalSize<u32>,
    ) -> Self {
        let (render_window_position, scale) =
            surface::estimate_render_window_position_scale(surface_size, scale, render_window_size);
        let render_window = (render_window_position, render_window_size);
        Self {
            render_window,
            scale,
        }
    }
}

impl EventContext<PhysicalPosition<f64>> for SoftEventContext {
    type SurfaceSpace = Option<PhysicalPosition<u32>>;

    fn estimate_surface_space(&self, event_space: PhysicalPosition<f64>) -> Self::SurfaceSpace {
        let PhysicalPosition { x, y } = event_space;
        let (x, y) = (x as u32, y as u32);
        if x > self.render_window.0.x && y > self.render_window.0.y {
            let (x, y) = (
                (x - self.render_window.0.x) / self.scale,
                (y - self.render_window.0.y) / self.scale,
            );
            if x < self.render_window.1.width && y < self.render_window.1.height {
                return Some(PhysicalPosition::new(x, y));
            }
        }
        None
    }
}

/// Event control structure.
pub struct SoftEventControl<'a> {
    event_loop: &'a ActiveEventLoop,
}

impl SoftEventControl<'_> {
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
    /// SoftBuffer error.
    SoftBufferError(softbuffer::SoftBufferError),
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

impl From<softbuffer::SoftBufferError> for Error {
    fn from(value: softbuffer::SoftBufferError) -> Self {
        Self::SoftBufferError(value)
    }
}
