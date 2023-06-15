use std::num::NonZeroU32;
use std::time::Duration;

use instant::Instant;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use self::config::Config;
use self::context::Context;
use self::input::Input;
use self::root::Root;
use self::setup::Setup;
use self::sound_system::SoundSystem;
use crate::visual::color::Converter;
use crate::visual::Image;

/// General application config.
pub mod config;
/// Context provided by the application during the `update`.
pub mod context;
/// User input handler.
pub mod input;
/// The root node of the devotee app.
pub mod root;
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
    constructor: Constructor<Cfg::Root, Cfg>,
    converter: Cfg::Converter,
    sound_system: Option<SoundSystem>,
    inner: Inner<Cfg>,
    input: Cfg::Input,
}

struct Inner<Cfg>
where
    Cfg: Config,
{
    window: window::Window,
    update_delay: Duration,
    render_target: Cfg::RenderTarget,
    pause_on_focus_lost: bool,
}

impl<Cfg> App<Cfg>
where
    Cfg: Config,
    Cfg::Converter: Converter<Palette = <Cfg::RenderTarget as Image>::Pixel>,
    Cfg::RenderTarget: Image,
{
    /// Create an app with given `setup`.
    pub fn with_setup(setup: Setup<Cfg>) -> Option<Self> {
        let event_loop = EventLoop::new();
        let window = window::Window::with_setup(&event_loop, &setup)?;
        let update_delay = setup.update_delay;
        let input = setup.input;
        let render_target = setup.render_target;
        let converter = Cfg::converter();
        let sound_system = SoundSystem::try_new();
        let constructor = setup.constructor;
        let pause_on_focus_lost = setup.pause_on_focus_lost;
        Some(Self {
            event_loop,
            constructor,
            converter,
            sound_system,
            inner: Inner {
                window,
                update_delay,
                render_target,
                pause_on_focus_lost,
            },
            input,
        })
    }
}

impl<Cfg> App<Cfg>
where
    Cfg: 'static + Config,
    Cfg::Root: Root<Cfg>,
    Cfg::Converter: Converter<Palette = <Cfg::RenderTarget as Image>::Pixel>,
    Cfg::Input: Input,
    for<'a> &'a Cfg::RenderTarget: IntoIterator<Item = &'a <Cfg::RenderTarget as Image>::Pixel>,
{
    /// Start the application event loop.
    pub fn run(self) {
        let app = self;
        let mut context = Context {
            delta: app.inner.update_delay,
            input: app.input,
            shall_stop: false,
            window_commands: Vec::new(),
            sound_system: app.sound_system,
            converter: app.converter,
        };

        let mut node = (app.constructor)(&mut context);
        if context.shall_stop() {
            return;
        }

        let event_loop = app.event_loop;
        let mut app = app.inner;
        let mut paused = false;

        app.window.apply(&mut context.window_commands);

        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(StartCause::Init) => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + app.update_delay);
            }
            Event::NewEvents(StartCause::ResumeTimeReached {
                requested_resume, ..
            }) => {
                *control_flow = ControlFlow::WaitUntil(requested_resume + app.update_delay);
                if !paused {
                    node.update(&mut context);

                    if context.shall_stop() {
                        *control_flow = ControlFlow::Exit;
                    }
                    app.window.apply(&mut context.window_commands);

                    context.input.next_frame();
                    if let Some(sound_system) = &mut context.sound_system {
                        sound_system.clean_up_sinks();
                    }
                }
                app.window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                node.render(&mut app.render_target);
                if app
                    .window
                    .draw_image(&app.render_target, &context.converter)
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                }
                app.window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                if let Some(event) = context.input.consume_window_event(event, &app.window) {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(size) => {
                            if let (Some(width), Some(height)) =
                                (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                            {
                                if app.window.resize_surface(width, height).is_err() {
                                    *control_flow = ControlFlow::Exit;
                                }
                            }
                        }
                        WindowEvent::Focused(focused) if app.pause_on_focus_lost => {
                            paused = !focused;
                            if paused {
                                context.sound_system.as_ref().map(SoundSystem::pause);
                            } else {
                                context.sound_system.as_ref().map(SoundSystem::resume);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        });
    }
}
