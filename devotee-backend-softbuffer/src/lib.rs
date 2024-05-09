#![deny(missing_docs)]

//! [Softbuffer](https://crates.io/crates/softbuffer)-based backend for the devotee project.

use std::num::TryFromIntError;
use std::rc::Rc;
use std::time::{Duration, Instant};

use devotee_backend::{
    Application, Context, Converter, EventContext, Middleware, RenderSurface, RenderTarget,
};
use softbuffer::{Buffer, SoftBufferError, Surface};
use winit::dpi::PhysicalSize;
use winit::error::{EventLoopError, OsError};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub use winit;

type Buf<'a> = Buffer<'a, Rc<Window>, Rc<Window>>;

/// Backend based on the [Softbuffer](https://crates.io/crates/softbuffer) project.
pub struct SoftBackend {
    window: Rc<Window>,
    event_loop: EventLoop<()>,
}

impl SoftBackend {
    /// Create new backend instance with desired window title.
    pub fn try_new(title: &str) -> Result<Self, Error> {
        let event_loop = EventLoop::new()?;
        let window = Rc::new(WindowBuilder::new().with_title(title).build(&event_loop)?);
        Ok(Self { window, event_loop })
    }
}

impl SoftBackend {
    /// Run this backend to completion.
    pub fn run<App, Mid, Rend, Data, Conv>(
        self,
        app: App,
        middleware: Mid,
        update_delay: Duration,
    ) -> Result<(), Error>
    where
        App: for<'a> Application<
            'a,
            <Mid as Middleware<'a, SoftControl>>::Init,
            <Mid as Middleware<'a, SoftControl>>::Context,
            Rend,
            Conv,
        >,
        Mid: for<'a> Middleware<
            'a,
            SoftControl,
            Event = WindowEvent,
            EventContext = &'a Window,
            Surface = Buf<'a>,
            RenderTarget = SoftRenderTarget<'a, Rend>,
        >,
        Rend: RenderSurface<Data = Data>,
        Conv: Converter<Data = Data>,
    {
        let mut app = app;
        let mut middleware = middleware;

        let window = self.window;

        let context = softbuffer::Context::new(window.clone())?;
        let mut surface = Surface::new(&context, window.clone())?;

        let mut control = SoftControl {
            should_quit: false,
            window: window.clone(),
        };
        let init = middleware.init(&mut control);
        app.init(init);

        surface.resize(
            window.inner_size().width.try_into()?,
            window.inner_size().height.try_into()?,
        )?;

        self.event_loop
            .set_control_flow(ControlFlow::WaitUntil(Instant::now() + update_delay));
        self.event_loop.run(move |event, elwt| {
            let mut control = SoftControl {
                should_quit: false,
                window: window.clone(),
            };

            match event {
                Event::NewEvents(StartCause::ResumeTimeReached {
                    requested_resume, ..
                }) => {
                    let context = middleware.update(&mut control, update_delay);
                    app.update(context);
                    elwt.set_control_flow(ControlFlow::WaitUntil(requested_resume + update_delay));
                    window.request_redraw();
                }
                Event::WindowEvent { event, .. } => {
                    if let Some(event) = middleware.handle_event(event, &window, &mut control) {
                        match event {
                            WindowEvent::Resized(size) => {
                                let width = size.width.try_into();
                                let height = size.height.try_into();
                                if let (Ok(width), Ok(height)) = (width, height) {
                                    let _ = surface.resize(width, height);
                                }
                            }
                            WindowEvent::RedrawRequested => {
                                if let Ok(buf) = surface.buffer_mut() {
                                    let mut render_target = middleware.render(buf);
                                    let surface = <SoftRenderTarget<'_, Rend> as RenderTarget<
                                        Conv,
                                    >>::render_surface_mut(
                                        &mut render_target
                                    );
                                    app.render(surface);
                                    let _ = devotee_backend::RenderTarget::present(
                                        render_target,
                                        app.converter(),
                                    );
                                }
                                window.request_redraw();
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            }

            if control.should_quit {
                elwt.exit();
            }
        })?;

        Ok(())
    }
}

/// Default Middleware for the Softbuffer backend.
pub struct SoftMiddleware<RenderSurface, Input> {
    background_color: u32,
    buffer_dimensions: (usize, usize),
    render_surface: RenderSurface,
    input: Input,
    default_scale: u32,
}

impl<RenderSurface, Input> SoftMiddleware<RenderSurface, Input>
where
    RenderSurface: devotee_backend::RenderSurface,
{
    /// Create new middleware instance with desired render surface and input handler.
    pub fn new(render_surface: RenderSurface, input: Input) -> Self {
        let buffer_dimensions = (render_surface.width(), render_surface.height());
        let background_color = 0;
        let default_scale = 1;
        Self {
            background_color,
            buffer_dimensions,
            render_surface,
            input,
            default_scale,
        }
    }

    /// Set default scale for the window.
    ///
    /// # Panics
    /// Panics if `default_scale` is zero.
    pub fn with_default_scale(self, default_scale: u32) -> Self {
        assert_ne!(default_scale, 0, "Default scale can't be zero");
        Self {
            default_scale,
            ..self
        }
    }

    /// Set background color for the unoccupied space.
    pub fn with_background_color(self, background_color: u32) -> Self {
        Self {
            background_color,
            ..self
        }
    }
}

impl<'a, RenderSurface, Input> Middleware<'a, SoftControl> for SoftMiddleware<RenderSurface, Input>
where
    RenderSurface: devotee_backend::RenderSurface,
    RenderSurface: 'a,
    Input: 'a + devotee_backend::Input<'a, SoftEventContext<'a>, Event = WindowEvent>,
{
    type Event = WindowEvent;
    type EventContext = &'a Window;
    type Surface = Buf<'a>;
    type Init = SoftInit<'a>;
    type Context = SoftContext<'a, Input>;
    type RenderTarget = SoftRenderTarget<'a, RenderSurface>;

    fn init(&'a mut self, control: &'a mut SoftControl) -> Self::Init {
        let dimensions = (
            self.render_surface.width() as u32,
            self.render_surface.height() as u32,
        );
        control
            .window
            .set_min_inner_size(Some(PhysicalSize::new(dimensions.0, dimensions.1)));
        let _ = control.window.request_inner_size(PhysicalSize::new(
            dimensions.0 * self.default_scale,
            dimensions.1 * self.default_scale,
        ));
        let actual_dimensions = control.window.inner_size();
        self.buffer_dimensions = (
            actual_dimensions.width as usize,
            actual_dimensions.height as usize,
        );

        SoftInit { control }
    }

    fn update(&'a mut self, control: &'a mut SoftControl, delta: Duration) -> Self::Context {
        let input = &mut self.input;
        SoftContext {
            control,
            delta,
            input,
        }
    }

    fn handle_event(
        &mut self,
        event: Self::Event,
        event_context: Self::EventContext,
        control: &mut SoftControl,
    ) -> Option<Self::Event> {
        let context = SoftEventContext {
            window: event_context,
            resolution: (
                self.render_surface.width() as u32,
                self.render_surface.height() as u32,
            ),
        };

        if let Some(event) = self.input.handle_event(event, &context) {
            match event {
                WindowEvent::CloseRequested => {
                    control.shutdown();
                }
                WindowEvent::Resized(internal_size) => {
                    self.buffer_dimensions =
                        (internal_size.width as usize, internal_size.height as usize);
                }
                _ => {}
            }

            Some(event)
        } else {
            None
        }
    }

    fn render(&'a mut self, surface: Self::Surface) -> Self::RenderTarget {
        let background_color = self.background_color;
        let buffer_dimensions = self.buffer_dimensions;
        let render_surface = &mut self.render_surface;
        SoftRenderTarget {
            background_color,
            buffer_dimensions,
            render_surface,
            buffer: surface,
        }
    }
}

/// Default Init for the Softbuffer backend.
pub struct SoftInit<'a> {
    control: &'a mut SoftControl,
}

impl<'a> SoftInit<'a> {
    /// Get reference to `SoftControl`.
    pub fn control(&self) -> &SoftControl {
        self.control
    }

    /// Get mutable reference to `SoftControl`.
    pub fn control_mut(&mut self) -> &mut SoftControl {
        self.control
    }
}

/// Default Context for the Softbuffer backend.
pub struct SoftContext<'a, Input>
where
    Input: devotee_backend::Input<'a, SoftEventContext<'a>>,
{
    control: &'a mut SoftControl,
    input: &'a mut Input,
    delta: Duration,
}

impl<'a, Input> SoftContext<'a, Input>
where
    Input: devotee_backend::Input<'a, SoftEventContext<'a>>,
{
    /// Get reference to `SoftControl`.
    pub fn control(&self) -> &SoftControl {
        self.control
    }

    /// Get mutable reference to `SoftControl`.
    pub fn control_mut(&mut self) -> &mut SoftControl {
        self.control
    }
}

impl<'a, Input> Context<'a, Input> for SoftContext<'a, Input>
where
    Input: devotee_backend::Input<'a, SoftEventContext<'a>>,
{
    fn input(&self) -> &Input {
        self.input
    }

    fn delta(&self) -> Duration {
        self.delta
    }

    fn shutdown(&mut self) {
        self.control.shutdown();
    }
}

impl<'a, Input> Drop for SoftContext<'a, Input>
where
    Input: devotee_backend::Input<'a, SoftEventContext<'a>>,
{
    fn drop(&mut self) {
        self.input.tick();
    }
}

/// Default Render Target for the Softbuffer backend.
pub struct SoftRenderTarget<'a, RenderSurface> {
    background_color: u32,
    buffer_dimensions: (usize, usize),
    render_surface: &'a mut RenderSurface,
    buffer: Buf<'a>,
}

impl<'a, RenderSurface, Converter> RenderTarget<Converter> for SoftRenderTarget<'a, RenderSurface>
where
    RenderSurface: devotee_backend::RenderSurface,
    Converter: devotee_backend::Converter<Data = RenderSurface::Data>,
{
    type RenderSurface = RenderSurface;
    type PresentError = SoftBufferError;

    fn render_surface(&self) -> &Self::RenderSurface {
        self.render_surface
    }

    fn render_surface_mut(&mut self) -> &mut Self::RenderSurface {
        self.render_surface
    }

    fn present(mut self, converter: Converter) -> Result<(), Self::PresentError> {
        let render_surface_dimensions = (self.render_surface.width(), self.render_surface.height());

        let scale_x = self.buffer_dimensions.0 / render_surface_dimensions.0;
        let scale_y = self.buffer_dimensions.1 / render_surface_dimensions.1;

        let minimal_scale = scale_x.min(scale_y);

        self.buffer.fill(self.background_color);
        if minimal_scale >= 1 {
            let start_x =
                (self.buffer_dimensions.0 - render_surface_dimensions.0 * minimal_scale) / 2;
            let start_y =
                (self.buffer_dimensions.1 - render_surface_dimensions.1 * minimal_scale) / 2;

            for y in 0..render_surface_dimensions.1 {
                for x in 0..render_surface_dimensions.0 {
                    let pixel_color = self.render_surface.data(x, y);
                    let pixel_value = converter.convert(x, y, pixel_color);
                    for iy in 0..minimal_scale {
                        let index = (start_x + x * minimal_scale)
                            + (iy + start_y + y * minimal_scale) * self.buffer_dimensions.0;
                        self.buffer[index..index + minimal_scale].fill(pixel_value);
                    }
                }
            }
        }

        self.buffer.present()
    }
}

/// Default Control instance for the Softbuffer backend.
pub struct SoftControl {
    should_quit: bool,
    window: Rc<Window>,
}

impl SoftControl {
    /// Tell backend to shut down.
    pub fn shutdown(&mut self) -> &mut Self {
        self.should_quit = true;
        self
    }

    /// Get reference to the underlying window.
    pub fn window_ref(&self) -> &Window {
        &self.window
    }
}

/// Default Event Context for the Softbuffer backend.
pub struct SoftEventContext<'a> {
    window: &'a Window,
    resolution: (u32, u32),
}

impl<'a> EventContext for SoftEventContext<'a> {
    fn position_into_render_surface_space(
        &self,
        position: (f32, f32),
    ) -> Result<(i32, i32), (i32, i32)> {
        let size = self.window.inner_size();
        let scale_x = size.width / self.resolution.0;
        let scale_y = size.height / self.resolution.1;

        let minimal_scale = scale_x.min(scale_y);

        if minimal_scale < 1 {
            Err((0, 0))
        } else {
            let position = (position.0 as i32, position.1 as i32);
            let start_x = ((size.width - self.resolution.0 * minimal_scale) / 2) as i32;
            let start_y = ((size.height - self.resolution.1 * minimal_scale) / 2) as i32;

            let position = (
                (position.0 - start_x) / minimal_scale as i32,
                (position.1 - start_y) / minimal_scale as i32,
            );

            if position.0 < 0
                || position.0 >= self.resolution.0 as i32
                || position.1 < 0
                || position.1 >= self.resolution.1 as i32
            {
                Err(position)
            } else {
                Ok(position)
            }
        }
    }
}

/// Softbuffer backend error enumeration.
#[derive(Debug)]
pub enum Error {
    /// Winit event loop error.
    WinitEventLoopError(EventLoopError),

    /// Winit OS error.
    WinitOsError(OsError),

    /// Softbuffer render error.
    SoftbufferError(SoftBufferError),

    /// Window resolution retrieval error.
    WindowResolutionError(TryFromIntError),
}

impl From<EventLoopError> for Error {
    fn from(value: EventLoopError) -> Self {
        Self::WinitEventLoopError(value)
    }
}

impl From<OsError> for Error {
    fn from(value: OsError) -> Self {
        Self::WinitOsError(value)
    }
}

impl From<SoftBufferError> for Error {
    fn from(value: SoftBufferError) -> Self {
        Self::SoftbufferError(value)
    }
}

impl From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        Self::WindowResolutionError(value)
    }
}
