use std::time::Duration;

pub trait Middleware<'a, Control> {
    type Event;
    type EventContext;
    type Surface;
    type Context;
    type RenderTarget;

    fn init(&'a mut self, control: &'a mut Control);
    fn update(&'a mut self, control: &'a mut Control, delta: Duration) -> Self::Context;
    fn handle_event(
        &mut self,
        event: Self::Event,
        event_context: Self::EventContext,
        control: &mut Control,
    ) -> Option<Self::Event>;
    fn render(&'a mut self, surface: Self::Surface) -> Self::RenderTarget;
}

pub trait Application<'a, Context, RenderSurface, Converter> {
    fn update(&mut self, context: Context);
    fn render(&self, render_surface: &mut RenderSurface);
    fn converter(&self) -> Converter;
    fn pause(&mut self) {}
    fn resume(&mut self) {}
}

pub trait RenderSurface {
    type Data;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn data(&self, x: usize, y: usize) -> Self::Data;
}

pub trait Converter {
    type Data;
    fn convert(&self, x: usize, y: usize, data: Self::Data) -> u32;
}

pub trait RenderTarget<Converter> {
    type RenderSurface;
    type PresentError;
    fn render_surface(&self) -> &Self::RenderSurface;
    fn render_surface_mut(&mut self) -> &mut Self::RenderSurface;
    fn present(self, converter: Converter) -> Result<(), Self::PresentError>;
}

pub trait Context<'a, Input> {
    fn input(&self) -> &Input;
    fn delta(&self) -> Duration;
    fn shutdown(&mut self);
}

pub trait Input<'a, EventContext> {
    type Event;

    fn handle_event(
        &mut self,
        event: Self::Event,
        event_context: &EventContext,
    ) -> Option<Self::Event>;
    fn tick(&mut self);
}

#[cfg(feature = "input-context")]
pub trait EventContext {
    fn position_into_render_surface_space(
        &self,
        position: (f32, f32),
    ) -> Result<(i32, i32), (i32, i32)>;
}
