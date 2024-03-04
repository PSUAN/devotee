use std::rc::Rc;

use rodio::source::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink};

pub use rodio;

pub type Sound = Rc<Sink>;

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
