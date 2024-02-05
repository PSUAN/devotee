use std::num::NonZeroU32;
use std::time::Duration;

use devotee_backend::winit::event::{Event, StartCause, WindowEvent};
use devotee_backend::winit::event_loop::{ControlFlow, EventLoop};
use devotee_backend::{Backend, BackendImage};
use instant::Instant;

use self::context::Context;
use self::input::Input;
use self::root::{ExitPermission, Root};
use self::setup::Setup;
use self::sound_system::SoundSystem;
use crate::visual::color::Converter;
use crate::visual::Image;

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
/// Takes mutable reference to `Context` and provides constructed root node.
pub type Constructor<T, I> = Box<dyn FnOnce(&mut Context<I>) -> T>;

/// App is the root of the `devotee` project.
/// It handles `winit`'s event loop and render.
pub struct App<R, Bck>
where
    R: Root,
{
    event_loop: EventLoop<()>,
    constructor: Constructor<R, R::Input>,
    sound_system: Option<SoundSystem>,
    inner: Inner<R, Bck>,
    input: R::Input,
}

struct Inner<R, Bck>
where
    R: Root,
{
    window: window::Window,
    update_delay: Duration,
    render_target: R::RenderTarget,
    pause_on_focus_lost: bool,
    backend: Bck,
}

impl<R, Bck> App<R, Bck>
where
    R: Root,
    R::Converter: Converter,
    R::RenderTarget: Image,
    Bck: Backend,
{
    /// Create an app with given `setup`.
    pub fn with_setup(setup: Setup<R>) -> Option<Self> {
        let event_loop = EventLoop::new();
        let window = window::Window::with_setup(&event_loop, &setup)?;
        let update_delay = setup.update_delay;
        let input = setup.input;
        let render_target = setup.render_target;
        let sound_system = SoundSystem::try_new();
        let constructor = setup.constructor;
        let pause_on_focus_lost = setup.pause_on_focus_lost;
        let backend = Bck::new(window.inner(), window.resolution().split(), setup.scale)?;
        Some(Self {
            event_loop,
            constructor,
            sound_system,
            inner: Inner {
                window,
                update_delay,
                render_target,
                pause_on_focus_lost,
                backend,
            },
            input,
        })
    }
}

impl<R, Bck> App<R, Bck>
where
    R: 'static + Root,
    R::Converter: Converter,
    R::Input: Input<Bck>,
    Bck: 'static + Backend,
    for<'a> R::RenderTarget: BackendImage<'a, <R::Converter as Converter>::Palette>,
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
                    .draw_image(
                        &mut app.backend,
                        &app.render_target,
                        node.converter(),
                        node.background_color(),
                    )
                    .is_none()
                {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::WindowEvent { event, .. } => {
                if let Some(event) =
                    context
                        .input
                        .consume_window_event(event, &app.window, &app.backend)
                {
                    match event {
                        WindowEvent::CloseRequested => {
                            if let ExitPermission::Allow = node.handle_exit_request() {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        WindowEvent::Resized(size) => {
                            if let (Some(width), Some(height)) =
                                (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                            {
                                if app.backend.resize(width, height).is_none() {
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
