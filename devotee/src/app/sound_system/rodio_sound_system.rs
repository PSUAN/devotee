use std::rc::Rc;

use rodio::source::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink, StreamError};

pub use rodio;

/// Reference-counted `rodio` sink.
pub type Sound = Rc<Sink>;

/// Simple sound system implementation.
pub struct SoundSystem {
    // We are storing `OutputStream` instance to save it from being dropped and thus stopping sound.
    #[allow(dead_code)]
    output_stream: OutputStream,
    handle: OutputStreamHandle,
    sinks: Vec<Rc<Sink>>,
}

impl SoundSystem {
    /// Try creating new Sound System instance.
    pub fn try_new() -> Result<Self, StreamError> {
        let (output_stream, handle) = OutputStream::try_default()?;
        let sinks = Vec::new();
        Ok(Self {
            output_stream,
            handle,
            sinks,
        })
    }

    fn free_sink(&self) -> Option<Rc<Sink>> {
        if let Some(free_sink) = self.sinks.iter().find(|sink| sink.empty()) {
            Some(Rc::clone(free_sink))
        } else {
            Sink::try_new(&self.handle).ok().map(Rc::new)
        }
    }

    /// Play passed source and get `Sound` instance if playback start was successful.
    pub fn play(&mut self, source: Box<dyn Source<Item = f32> + Send>) -> Option<Sound> {
        if let Some(sink) = self.free_sink() {
            sink.append(source);
            self.sinks.push(sink.clone());
            Some(sink)
        } else {
            None
        }
    }

    /// Pause playback.
    pub fn pause(&self) {
        for sink in self.sinks.iter() {
            sink.pause();
        }
    }

    /// Resume playback.
    pub fn resume(&self) {
        for sink in self.sinks.iter() {
            sink.play();
        }
    }
}
