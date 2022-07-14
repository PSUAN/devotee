use crate::node::Node;
use crate::visual::canvas::Canvas;
use crate::visual::color::Converter;
use config::Config;
use context::UpdateContext;
use input::Input;
use pixels::Pixels;
use setup::Setup;
use sound_system::SoundSystem;
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
/// `rodio`-based sound system.
pub mod sound_system;
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
    sound_system: Option<SoundSystem>,
}

impl<Cfg> App<Cfg>
where
    Cfg: Config,
    Cfg::Palette: Copy,
    Cfg::Converter: Converter<Palette = Cfg::Palette>,
{
    /// Create an app with given setup.
    pub fn with_setup(setup: Setup<Cfg>) -> Option<Self> {
        let event_loop = EventLoop::new();
        let window = window::Window::with_setup(&event_loop, &setup)?;
        let node = setup.node;
        let update_delay = setup.update_delay;
        let input = Input::default();
        let canvas = Canvas::with_resolution(
            Cfg::background_color(),
            setup.resolution.x(),
            setup.resolution.y(),
        );
        let converter = Cfg::converter();
        let sound_system = SoundSystem::try_new();
        Some(Self {
            event_loop,
            inner: Inner {
                window,
                node,
                update_delay,
                input,
                canvas,
                converter,
                sound_system,
            },
        })
    }
}

impl<Cfg> App<Cfg>
where
    Cfg: 'static + Config,
    Cfg::Node: for<'a, 'b, 'c> Node<&'a mut UpdateContext<'b>, &'c mut Canvas<Cfg::Palette>>,
    Cfg::Converter: Converter<Palette = Cfg::Palette>,
    Cfg::Palette: Clone,
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
        let mut paused = false;

        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(StartCause::Init) => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + app.update_delay);
            }
            Event::NewEvents(StartCause::ResumeTimeReached {
                requested_resume, ..
            }) => {
                *control_flow = ControlFlow::WaitUntil(requested_resume + app.update_delay);
                if !paused {
                    let mut update =
                        UpdateContext::new(app.update_delay, &app.input, app.sound_system.as_mut());

                    app.node.update(&mut update);
                    if update.shall_stop() {
                        *control_flow = ControlFlow::Exit;
                    }
                    app.window.apply(update.extract_window_commands());
                    app.input.step();
                    if let Some(sound_system) = &mut app.sound_system {
                        sound_system.clean_up_sinks();
                    }
                }
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
            Event::WindowEvent {
                event: WindowEvent::Focused(focused),
                ..
            } => {
                paused = !focused;
                if paused {
                    app.sound_system.as_ref().map(SoundSystem::pause);
                } else {
                    app.sound_system.as_ref().map(SoundSystem::resume);
                }
            }
            _ => {}
        });
    }
}
