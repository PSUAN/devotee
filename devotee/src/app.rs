use std::time::Duration;

use devotee_backend::Application;

use self::sound_system::SoundSystem;

pub mod root;
pub mod sound_system;

pub struct App<Root> {
    root: Root,
    sound_system: Option<SoundSystem>,
}

impl<Root> App<Root> {
    pub fn new(root: Root) -> Self {
        let sound_system = SoundSystem::try_new();
        Self { root, sound_system }
    }
}

impl<'a, Root, Context, RenderSurface, Converter, In>
    Application<'a, Context, RenderSurface, Converter> for App<Root>
where
    Root: root::Root<RenderSurface = RenderSurface, Converter = Converter, Input = In>,
    Context: backend::Context<'a, In> + 'a,
    In: 'a,
{
    fn update(&mut self, mut context: Context) {
        let context = &mut context;
        let sound_system = self.sound_system.as_mut();
        let context = AppContext {
            sound_system,
            context,
        };
        self.root.update(context);
    }

    fn render(&self, render_surface: &mut RenderSurface) {
        self.root.render(render_surface);
    }

    fn converter(&self) -> Converter {
        self.root.converter()
    }

    fn pause(&mut self) {
        if let Some(s) = self.sound_system.as_mut() {
            s.pause()
        };
    }

    fn resume(&mut self) {
        if let Some(s) = self.sound_system.as_mut() {
            s.resume()
        }
    }
}

pub struct AppContext<'a, 'b, Input> {
    sound_system: Option<&'b mut SoundSystem>,
    context: &'b mut dyn backend::Context<'a, Input>,
}

impl<'a, 'b, Input> AppContext<'a, 'b, Input> {
    pub fn delta(&self) -> Duration {
        self.context.delta()
    }

    pub fn input(&self) -> &Input {
        self.context.input()
    }

    pub fn try_sound_system_mut(&mut self) -> Option<&mut SoundSystem> {
        self.sound_system.as_deref_mut()
    }

    pub fn shutdown(&mut self) -> &mut Self {
        self.context.shutdown();
        self
    }
}
