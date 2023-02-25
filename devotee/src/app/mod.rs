use std::time::Duration;

use instant::Instant;
use pixels::Pixels;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use self::config::Config;
use self::context::Context;
use self::input::Input;
use self::setup::Setup;
use self::sound_system::SoundSystem;
use crate::node::Node;
use crate::visual::canvas::Canvas;
use crate::visual::color::Converter;

/// General application config.
pub mod config;
/// Context provided by the application during the `update`.
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
/// Takes mutable reference to `Context` and provides constructed node.
pub type Constructor<T, U> = Box<dyn FnOnce(&mut Context<U>) -> T>;

/// App is the root of the `devotee` project.
/// It handles `winit`'s event loop and render.
pub struct App<Cfg>
where
    Cfg: Config,
{
    event_loop: EventLoop<()>,
    constructor: Constructor<Cfg::Node, Cfg>,
    inner: Inner<Cfg>,
    input: Cfg::Input,
}

struct Inner<Cfg>
where
    Cfg: Config,
{
    window: window::Window,
    update_delay: Duration,
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
    /// Create an app with given `setup`.
    pub fn with_setup(setup: Setup<Cfg>) -> Option<Self> {
        let event_loop = EventLoop::new();
        let window = window::Window::with_setup(&event_loop, &setup)?;
        let update_delay = setup.update_delay;
        let input = setup.input;
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
                canvas,
                converter,
                sound_system,
                pause_on_focus_lost,
            },
            input,
        })
    }
}

impl<Cfg> App<Cfg>
where
    Cfg: 'static + Config,
    Cfg::Node: for<'a, 'b> Node<&'a mut Context<Cfg>, &'b mut Canvas<Cfg::Palette>>,
    Cfg::Converter: Converter<Palette = Cfg::Palette>,
    Cfg::Palette: Clone,
    Cfg::Input: Input,
{
    fn convert(pixels: &mut Pixels, canvas: &Canvas<Cfg::Palette>, converter: &Cfg::Converter) {
        for (pixel, palette) in pixels
            .get_frame_mut()
            .chunks_exact_mut(4)
            .zip(canvas.iter())
        {
            let color = converter.convert(palette);
            pixel.copy_from_slice(&color);
        }
    }

    /// Start the application event loop.
    pub fn run(self) {
        let mut app = self;
        let mut update = Context::new(
            app.inner.update_delay,
            app.input,
            app.inner.sound_system.take(),
        );

        let mut node = (app.constructor)(&mut update);
        if update.shall_stop() {
            return;
        }
        let (sound_system, input, commands) = update.decompose();

        let event_loop = app.event_loop;
        let mut app = app.inner;
        app.sound_system = sound_system;
        let mut paused = false;
        let mut input = Some(input);

        app.window.apply(commands);

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::NewEvents(StartCause::Init) => {
                    *control_flow = ControlFlow::WaitUntil(Instant::now() + app.update_delay);
                }
                Event::NewEvents(StartCause::ResumeTimeReached {
                    requested_resume, ..
                }) => {
                    *control_flow = ControlFlow::WaitUntil(requested_resume + app.update_delay);
                    if !paused {
                        // SAFETY: We are certain that we did not forget to put input back.
                        let mut update = Context::new(
                            app.update_delay,
                            input.take().unwrap(),
                            app.sound_system.take(),
                        );

                        node.update(&mut update);
                        if update.shall_stop() {
                            *control_flow = ControlFlow::Exit;
                        }
                        let (sound_system, mut returned_input, window_commands) =
                            update.decompose();
                        app.window.apply(window_commands);
                        app.sound_system = sound_system;

                        returned_input.next_frame();
                        if let Some(sound_system) = &mut app.sound_system {
                            sound_system.clean_up_sinks();
                        }

                        input = Some(returned_input);
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
                Event::WindowEvent { event, .. } => {
                    // SAFETY: we believe that we did not forget to put input back.
                    if let Some(event) = input
                        .as_mut()
                        .unwrap()
                        .consume_window_event(event, app.window.pixels())
                    {
                        match event {
                            WindowEvent::CloseRequested => {
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::Resized(size) => {
                                app.window
                                    .pixels_mut()
                                    .resize_surface(size.width, size.height);
                            }
                            WindowEvent::Focused(focused) if app.pause_on_focus_lost => {
                                paused = !focused;
                                if paused {
                                    app.sound_system.as_ref().map(SoundSystem::pause);
                                } else {
                                    app.sound_system.as_ref().map(SoundSystem::resume);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        });
    }
}
