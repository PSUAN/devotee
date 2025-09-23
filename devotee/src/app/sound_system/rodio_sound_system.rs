use std::rc::Rc;

use rodio::{OutputStream, OutputStreamBuilder, Sink, Source, StreamError};

pub use rodio;

/// Reference-counted `rodio` sink.
pub type Sound = Rc<Sink>;

/// Simple sound system implementation.
pub struct SoundSystem {
    output_stream: OutputStream,
    sinks: Vec<Rc<Sink>>,
}

impl SoundSystem {
    /// Try creating new Sound System instance.
    pub fn try_new() -> Result<Self, StreamError> {
        let output_stream = OutputStreamBuilder::open_default_stream()?;
        let sinks = Vec::new();
        Ok(Self {
            output_stream,
            sinks,
        })
    }

    fn free_sink(&mut self) -> Rc<Sink> {
        if let Some(free_sink) = self.sinks.iter().find(|sink| sink.empty()) {
            Rc::clone(free_sink)
        } else {
            let sink = Rc::new(Sink::connect_new(self.output_stream.mixer()));
            self.sinks.push(Rc::clone(&sink));
            sink
        }
    }

    /// Play passed source and get `Sound` instance.
    pub fn play(&mut self, source: Box<dyn Source<Item = f32> + Send>) -> Sound {
        let sink = self.free_sink();
        sink.append(source);
        sink
    }

    /// Pause for all sounds playback.
    pub fn pause(&self) {
        for sink in self.sinks.iter() {
            sink.pause();
        }
    }

    /// Resume playback for all sounds.
    pub fn resume(&self) {
        for sink in self.sinks.iter() {
            sink.play();
        }
    }
}
