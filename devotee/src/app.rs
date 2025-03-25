use backend::middling::Application;

/// Application root specification.
pub mod root;

/// Sound system implementations.
pub mod sound_system;

/// Default Application implementation.
pub struct App<Root> {
    root: Root,
}

impl<Root> App<Root> {
    /// Create a new App instance.
    pub fn new(root: Root) -> Self {
        Self { root }
    }
}

impl<Root, Init, Context, Input, Surface> Application<Init, Context, Input, Surface> for App<Root>
where
    Root: root::Root<Init, Context, Input, Surface>,
{
    fn init(&mut self, init: &mut Init) {
        self.root.init(init);
    }

    fn update(&mut self, context: &mut Context, input: &Input) {
        self.root.update(context, input);
    }

    fn render(&mut self, surface: &mut Surface) {
        self.root.render(surface);
    }
}
