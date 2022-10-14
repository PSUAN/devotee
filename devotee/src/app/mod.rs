use crate::node::Node;
use crate::visual::canvas::Canvas;
use crate::visual::color::Converter;
use config::Config;
use context::UpdateContext;
use input::Input;
#[cfg(target_arch = "wasm32")]
use instant::Instant;
use pixels::Pixels;
use setup::Setup;
use sound_system::SoundSystem;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
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

/// Node constructor.
/// Takes mutable reference to `UpdateContext` and provides new node.
pub type Constructor<T> = Box<dyn FnOnce(&mut UpdateContext) -> T>;

/// App is the root of the `devotee` project.
/// It handles `winit`'s event loop and render.
pub struct App<Cfg>
where
    Cfg: Config,
{
    event_loop: EventLoop<()>,
    constructor: Constructor<Cfg::Node>,
    inner: Inner<Cfg>,
}

struct Inner<Cfg>
where
    Cfg: Config,
{
    window: window::Window,
    update_delay: Duration,
    input: Input,
    canvas: Canvas<Cfg::Palette>,
    converter: Cfg::Converter,
    sound_system: Option<SoundSystem>,
    pause_on_focus_lost: bool,
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
        let update_delay = setup.update_delay;
        let input = Input::default();
        let canvas = Canvas::with_resolution(
            Cfg::background_color(),
            setup.resolution.x(),
            setup.resolution.y(),
        );
        let converter = Cfg::converter();
        let sound_system = SoundSystem::try_new();
        let constructor = setup.constructor;
        let pause_on_focus_lost = setup.pause_on_focus_lost;
        Some(Self {
            event_loop,
            constructor,
            inner: Inner {
                window,
                update_delay,
                input,
                canvas,
                converter,
                sound_system,
                pause_on_focus_lost,
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
        let mut app = self;
        let mut update = UpdateContext::new(
            app.inner.update_delay,
            &app.inner.input,
            app.inner.sound_system.as_mut(),
        );

        let mut node = (app.constructor)(&mut update);
        if update.shall_stop() {
            return;
        }

        let event_loop = app.event_loop;
        let mut app = app.inner;
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

                    node.update(&mut update);
                    if update.shall_stop() {
                        *control_flow = ControlFlow::Exit;
                    }
                    app.window.apply(update.extract_window_commands());
                    app.input.step();
                    if let Some(sound_system) = &mut app.sound_system {
                        sound_system.clean_up_sinks();
                    }
                }
                app.window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                node.render(&mut app.canvas);
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
            } if app.pause_on_focus_lost => {
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
