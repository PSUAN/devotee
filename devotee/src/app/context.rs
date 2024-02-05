use std::time::Duration;

use super::sound_system::SoundSystem;
use super::window::{Window, WindowCommand};

/// Context that stores various update-related data.
pub struct Context<Input> {
    pub(super) delta: Duration,
    pub(super) input: Input,
    pub(super) shall_stop: bool,
    pub(super) window_commands: Vec<WindowCommand>,
    pub(super) sound_system: Option<SoundSystem>,
}

impl<Input> Context<Input> {
    /// Get `Duration` of simulation step.
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Get reference to the `Input` structure.
    pub fn input(&self) -> &Input {
        &self.input
    }

    /// Tell the `App` to stop execution.
    pub fn shutdown(&mut self) {
        self.shall_stop = true;
    }

    pub(super) fn shall_stop(&self) -> bool {
        self.shall_stop
    }

    /// Enqueue command to be executed on app's window.
    pub fn add_window_command<F: 'static + FnOnce(&mut Window)>(&mut self, command: F) {
        self.window_commands.push(Box::new(command))
    }

    /// Get optional reference to the `SoundSystem`.
    pub fn sound_system(&mut self) -> Option<&SoundSystem> {
        self.sound_system.as_ref()
    }

    /// Get optional mutable reference to the `SoundSystem`.
    pub fn sound_system_mut(&mut self) -> Option<&mut SoundSystem> {
        self.sound_system.as_mut()
    }
}
