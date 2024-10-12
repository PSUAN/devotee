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

impl<'a, Root, Init, Context, RenderSurface, Converter>
    Application<'a, Init, Context, RenderSurface, Converter> for App<Root>
where
    Root: root::Root<Init, Context, RenderSurface = RenderSurface, Converter = Converter>,
{
    fn init(&mut self, mut init: Init) {
        self.root.init(&mut init);
    }

    fn update(&mut self, mut context: Context) {
        let context = &mut context;
        self.root.update(context);
    }

    fn render(&mut self, render_surface: &mut RenderSurface) {
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
