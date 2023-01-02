use rodio::source::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::mem;
use std::rc::Rc;

pub use rodio;

/// `rodio`'s `Sink` wrapped in reference counter.
pub type Sound = Rc<Sink>;

/// `rodio`-based sound system,
pub struct SoundSystem {
    // We are storing `OutputStream` instance to save it from being dropped and thus stopping sound.
    #[allow(dead_code)]
    output_stream: OutputStream,
    handle: OutputStreamHandle,
    sinks: Vec<Rc<Sink>>,
}

impl SoundSystem {
    pub(crate) fn try_new() -> Option<Self> {
        let (output_stream, handle) = OutputStream::try_default().ok()?;
        let sinks = Vec::new();
        Some(Self {
            output_stream,
            handle,
            sinks,
        })
    }

    fn sink(&self) -> Option<Sink> {
        Sink::try_new(&self.handle).ok()
    }

    /// Play given sound.
    /// Returns `None` in fail case.
    /// The returned sink may be used to stop sound.
    pub fn play(&mut self, source: Box<dyn Source<Item = f32> + Send>) -> Option<Rc<Sink>> {
        if let Some(sink) = self.sink() {
            sink.append(source);
            let sink = Rc::new(sink);
            self.sinks.push(sink.clone());
            Some(sink)
        } else {
            None
        }
    }

    pub(super) fn clean_up_sinks(&mut self) {
        self.sinks = mem::take(&mut self.sinks)
            .into_iter()
            .filter(|sink| !sink.empty())
            .collect();
    }

    pub(super) fn pause(&self) {
        for sink in self.sinks.iter() {
            sink.pause();
        }
    }

    pub(super) fn resume(&self) {
        for sink in self.sinks.iter() {
            sink.play();
        }
    }
}
