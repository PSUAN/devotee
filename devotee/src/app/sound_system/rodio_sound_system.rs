use std::rc::Rc;

use rodio::{DeviceSinkBuilder, DeviceSinkError, MixerDeviceSink, Player, Source};

pub use rodio;

/// Reference-counted `rodio` player.
pub type Sound = Rc<Player>;

/// Simple sound system implementation.
pub struct SoundSystem {
    output_stream: MixerDeviceSink,
    players: Vec<Sound>,
}

impl SoundSystem {
    /// Try creating new Sound System instance.
    pub fn try_new() -> Result<Self, DeviceSinkError> {
        let output_stream = DeviceSinkBuilder::open_default_sink()?;
        let players = Vec::new();
        Ok(Self {
            output_stream,
            players,
        })
    }

    fn free_player(&mut self) -> Sound {
        if let Some(free_player) = self.players.iter_mut().find(|player| player.empty()) {
            *free_player = Rc::new(Player::connect_new(self.output_stream.mixer()));
            Rc::clone(free_player)
        } else {
            let player = Rc::new(Player::connect_new(self.output_stream.mixer()));
            self.players.push(Rc::clone(&player));
            player
        }
    }

    /// Play passed source and get `Sound` instance.
    pub fn play(&mut self, source: Box<dyn Source<Item = f32> + Send>) -> Sound {
        let player = self.free_player();
        player.append(source);
        player
    }

    /// Pause for all sounds playback.
    pub fn pause(&self) {
        for player in self.players.iter() {
            player.pause();
        }
    }

    /// Resume playback for all sounds.
    pub fn resume(&self) {
        for player in self.players.iter() {
            player.play();
        }
    }
}
