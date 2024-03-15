use std::time::Duration;

use devotee_backend::Application;

/// Application root specification.
pub mod root;

/// Sound system implementations.
pub mod sound_system;

/// Default Application implementation.
pub struct App<Root> {
    root: Root,
}

impl<Root> App<Root> {
    /// Create new App with passed root.
    pub fn new(root: Root) -> Self {
        Self { root }
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
        let context = AppContext { context };
        self.root.update(context);
    }

    fn render(&self, render_surface: &mut RenderSurface) {
        self.root.render(render_surface);
    }

    fn converter(&self) -> Converter {
        self.root.converter()
    }

    fn pause(&mut self) {
        self.root.pause();
    }

    fn resume(&mut self) {
        self.root.resume();
    }
}

/// Context passed to the Root instance during update call.
pub struct AppContext<'a, 'b, Input> {
    context: &'a mut dyn backend::Context<'b, Input>,
}

impl<'a, 'b, Input> AppContext<'a, 'b, Input> {
    /// Get simulation time passed since the previous update.
    pub fn delta(&self) -> Duration {
        self.context.delta()
    }

    /// Get reference to the input.
    pub fn input(&self) -> &Input {
        self.context.input()
    }

    /// Tell the application to shut down.
    pub fn shutdown(&mut self) -> &mut Self {
        self.context.shutdown();
        self
    }
}
