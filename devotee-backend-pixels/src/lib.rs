use std::num::TryFromIntError;
use std::rc::Rc;
use std::time::{Duration, Instant};

use devotee_backend::{
    Application, Context, Converter, EventContext, Middleware, RenderSurface, RenderTarget,
};
use pixels::{Error as PixelsError, Pixels, PixelsBuilder, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::error::{EventLoopError, OsError};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct PixelsBackend {
    window: Rc<Window>,
    event_loop: EventLoop<()>,
}

impl PixelsBackend {
    pub fn try_new(title: &str) -> Result<Self, Error> {
        let event_loop = EventLoop::new()?;
        let window = Rc::new(WindowBuilder::new().with_title(title).build(&event_loop)?);
        Ok(Self { window, event_loop })
    }
}

impl PixelsBackend {
    pub fn run<App, Mid, Rend, Data, Conv, Ctx>(
        self,
        app: App,
        middleware: Mid,
        update_delay: Duration,
    ) -> Result<(), Error>
    where
        App: for<'a> Application<'a, Ctx, Rend, Conv>,
        Mid: for<'a> Middleware<
            'a,
            PixelsControl,
            Event = WindowEvent,
            EventContext = &'a Pixels,
            Surface = &'a mut Pixels,
            RenderTarget = PixelsRenderTarget<'a, Rend>,
            Context = Ctx,
        >,
        Rend: RenderSurface<Data = Data>,
        Conv: Converter<Data = Data>,
    {
        let mut app = app;
        let mut middleware = middleware;

        let window = self.window;

        let mut control = PixelsControl {
            should_quit: false,
            paused: None,
            window: window.clone(),
        };
        middleware.init(&mut control);

        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            PixelsBuilder::new(window_size.width, window_size.height, surface_texture)
                .enable_vsync(true)
                .build()?
        };

        self.event_loop
            .set_control_flow(ControlFlow::WaitUntil(Instant::now() + update_delay));
        self.event_loop.run(move |event, elwt| {
            let mut control = PixelsControl {
                should_quit: false,
                paused: None,
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
                    if let Some(event) = middleware.handle_event(event, &pixels, &mut control) {
                        match event {
                            WindowEvent::Resized(size) => {
                                let width = size.width;
                                let height = size.height;
                                let _ = pixels.resize_surface(width, height);
                            }
                            WindowEvent::RedrawRequested => {
                                let mut render_target = middleware.render(&mut pixels);
                                let surface = <PixelsRenderTarget<'_, Rend> as RenderTarget<
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
                            _ => (),
                        }
                    }
                }
                _ => (),
            }

            if control.should_quit {
                elwt.exit();
            }
            if let Some(paused) = control.paused {
                if paused {
                    app.pause();
                } else {
                    app.resume();
                }
            }
        })?;

        Ok(())
    }
}

pub struct PixelsMiddleware<RenderSurface, Input> {
    render_surface: RenderSurface,
    input: Input,
}

impl<RenderSurface, Input> PixelsMiddleware<RenderSurface, Input>
where
    RenderSurface: devotee_backend::RenderSurface,
{
    pub fn new(render_surface: RenderSurface, input: Input) -> Self {
        Self {
            render_surface,
            input,
        }
    }
}

impl<'a, RenderSurface, Input> Middleware<'a, PixelsControl>
    for PixelsMiddleware<RenderSurface, Input>
where
    RenderSurface: devotee_backend::RenderSurface,
    RenderSurface: 'a,
    Input: 'a + devotee_backend::Input<'a, PixelsEventContext<'a>, Event = WindowEvent>,
{
    type Event = WindowEvent;
    type EventContext = &'a Pixels;
    type Surface = &'a mut Pixels;
    type Context = PixelsContext<'a, Input>;
    type RenderTarget = PixelsRenderTarget<'a, RenderSurface>;

    fn init(&'a mut self, control: &'a mut PixelsControl) {
        let size = PhysicalSize::new(
            self.render_surface.width() as u32,
            self.render_surface.height() as u32,
        );
        control.window.set_min_inner_size(Some(size));
    }

    fn update(&'a mut self, control: &'a mut PixelsControl, delta: Duration) -> Self::Context {
        let input = &mut self.input;
        PixelsContext {
            control,
            delta,
            input,
        }
    }

    fn handle_event(
        &mut self,
        event: Self::Event,
        event_context: Self::EventContext,
        control: &mut PixelsControl,
    ) -> Option<Self::Event> {
        let context = PixelsEventContext {
            pixels: event_context,
        };

        if let Some(event) = self.input.handle_event(event, &context) {
            match event {
                WindowEvent::CloseRequested => {
                    control.shutdown();
                }
                WindowEvent::Focused(gained) => {
                    control.set_paused(!gained);
                }
                _ => {}
            }

            Some(event)
        } else {
            None
        }
    }

    fn render(&'a mut self, surface: Self::Surface) -> Self::RenderTarget {
        PixelsRenderTarget {
            render_surface: &mut self.render_surface,
            pixels: surface,
        }
    }
}

pub struct PixelsContext<'a, Input>
where
    Input: devotee_backend::Input<'a, PixelsEventContext<'a>>,
{
    control: &'a mut PixelsControl,
    input: &'a mut Input,
    delta: Duration,
}

impl<'a, Input> Context<'a, Input> for PixelsContext<'a, Input>
where
    Input: devotee_backend::Input<'a, PixelsEventContext<'a>>,
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

impl<'a, Input> Drop for PixelsContext<'a, Input>
where
    Input: devotee_backend::Input<'a, PixelsEventContext<'a>>,
{
    fn drop(&mut self) {
        self.input.tick();
    }
}

pub struct PixelsRenderTarget<'a, RenderSurface> {
    render_surface: &'a mut RenderSurface,
    pixels: &'a mut Pixels,
}

impl<'a, RenderSurface, Converter> RenderTarget<Converter> for PixelsRenderTarget<'a, RenderSurface>
where
    RenderSurface: devotee_backend::RenderSurface,
    Converter: devotee_backend::Converter<Data = RenderSurface::Data>,
{
    type RenderSurface = RenderSurface;
    type PresentError = PixelsError;

    fn render_surface(&self) -> &Self::RenderSurface {
        self.render_surface
    }

    fn render_surface_mut(&mut self) -> &mut Self::RenderSurface {
        self.render_surface
    }

    fn present(self, converter: Converter) -> Result<(), Self::PresentError> {
        self.pixels.resize_buffer(
            self.render_surface.width() as u32,
            self.render_surface.height() as u32,
        )?;

        for (y, line) in self
            .pixels
            .frame_mut()
            .chunks_exact_mut(self.render_surface.width() * 4)
            .enumerate()
        {
            for (x, pixel) in line.chunks_exact_mut(4).enumerate() {
                let pixel_color = self.render_surface.data(x, y);
                let pixel_value = converter.convert(x, y, pixel_color);
                let rgba = [
                    ((pixel_value & 0x00_ff_00_00) >> 16) as u8,
                    ((pixel_value & 0x00_00_ff_00) >> 8) as u8,
                    (pixel_value & 0x00_00_00_ff) as u8,
                    0xff,
                ];
                pixel.copy_from_slice(&rgba);
            }
        }
        self.pixels.render()
    }
}

pub struct PixelsControl {
    should_quit: bool,
    paused: Option<bool>,
    window: Rc<Window>,
}

impl PixelsControl {
    pub fn shutdown(&mut self) -> &mut Self {
        self.should_quit = true;
        self
    }

    fn set_paused(&mut self, paused: bool) -> &mut Self {
        self.paused = Some(paused);
        self
    }
}

pub struct PixelsEventContext<'a> {
    pixels: &'a Pixels,
}

impl<'a> EventContext for PixelsEventContext<'a> {
    fn position_into_render_surface_space(
        &self,
        position: (f32, f32),
    ) -> Result<(i32, i32), (i32, i32)> {
        self.pixels
            .window_pos_to_pixel(position)
            .map(|(x, y)| (x as i32, y as i32))
            .map_err(|(x, y)| (x as i32, y as i32))
    }
}

#[derive(Debug)]
pub enum Error {
    WinitEventLoopError(EventLoopError),
    WinitOsError(OsError),
    PixelsError(PixelsError),
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

impl From<PixelsError> for Error {
    fn from(value: PixelsError) -> Self {
        Self::PixelsError(value)
    }
}

impl From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        Self::WindowResolutionError(value)
    }
}
