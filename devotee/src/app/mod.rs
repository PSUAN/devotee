use crate::node::Node;
use crate::visual::canvas::Canvas;
use crate::visual::color::Converter;
use config::Config;
use context::UpdateContext;
use input::Input;
use pixels::Pixels;
use setup::Setup;
use std::time::{Duration, Instant};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

/// General application config.
pub mod config;
/// Context provided by the application.
pub mod context;
/// User input handler.
pub mod input;
/// Application launch setup.
pub mod setup;
/// Main application window.
pub mod window;

/// App is the root of the `devotee` project.
/// It handles `winit`'s event loop and render.
pub struct App<Cfg>
where
    Cfg: Config,
{
    event_loop: EventLoop<()>,
    inner: Inner<Cfg>,
}

struct Inner<Cfg>
where
    Cfg: Config,
{
    window: window::Window,
    node: Cfg::Node,
    update_delay: Duration,
    input: Input,
    canvas: Canvas<Cfg::Palette>,
    converter: Cfg::Converter,
}

impl<Cfg> App<Cfg>
where
    Cfg: Config,
    Cfg::Palette: Copy,
    Cfg::Converter: Converter<Palette = Cfg::Palette>,
{
    /// Create an app with given config.
    pub fn with_config(config: Setup<Cfg>) -> Option<Self> {
        let event_loop = EventLoop::new();
        let window = window::Window::with_setup(&event_loop, &config)?;
        let node = config.node;
        let update_delay = config.update_delay;
        let input = Input::default();
        let canvas = Canvas::with_resolution(
            Cfg::background_color(),
            config.resolution.x(),
            config.resolution.y(),
        );
        let converter = Cfg::converter();
        Some(Self {
            event_loop,
            inner: Inner {
                window,
                node,
                update_delay,
                input,
                canvas,
                converter,
            },
        })
    }
}

impl<Cfg> App<Cfg>
where
    Cfg: 'static + Config,
    Cfg::Node: Node<Update = UpdateContext, Render = Canvas<Cfg::Palette>>,
    Cfg::Converter: Converter<Palette = Cfg::Palette>,
    Cfg::Palette: Copy,
{
    fn convert(pixels: &mut Pixels, canvas: &Canvas<Cfg::Palette>, converter: &Cfg::Converter) {
        for (pixel, palette) in pixels.get_frame().chunks_exact_mut(4).zip(canvas.iter()) {
            let color = converter.convert(palette);
            pixel.copy_from_slice(&color);
        }
    }

    /// Start the application event loop.
    pub fn run(self) {
        let mut app = self.inner;
        let event_loop = self.event_loop;
        event_loop.run(
            move |event, _, control_flow: &mut ControlFlow| match event {
                Event::NewEvents(StartCause::Init) => {
                    *control_flow = ControlFlow::WaitUntil(Instant::now() + app.update_delay);
                }
                Event::NewEvents(StartCause::ResumeTimeReached {
                    requested_resume, ..
                }) => {
                    *control_flow = ControlFlow::WaitUntil(requested_resume + app.update_delay);

                    let mut update = UpdateContext::new(app.update_delay, app.input.clone());
                    {
                        app.node.update(&mut update);
                    }
                    if update.shall_stop() {
                        *control_flow = ControlFlow::Exit;
                    }
                    app.window.apply(update.extract_window_commands());
                    app.input.step();
                }
                Event::RedrawRequested(_) => {
                    app.node.render(&mut app.canvas);
                    Self::convert(app.window.pixels_mut(), &app.canvas, &app.converter);
                    if app.window.render().is_err() {
                        *control_flow = ControlFlow::Exit;
                    }
                    app.window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => app.input.register_key_event(input),
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    app.window
                        .pixels_mut()
                        .resize_surface(size.width, size.height);
                }
                _ => {}
            },
        );
    }
}
