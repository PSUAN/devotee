/// Generic root trait.
pub trait Root<Init, Context, Input, Surface> {
    /// Perform the initialization process.
    fn init(&mut self, init: &mut Init);

    /// Handle the update event.
    fn update(&mut self, context: &mut Context, input: &Input);

    /// Perform rendering on the passed image.
    fn render(&mut self, surface: &mut Surface);
}
