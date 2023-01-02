use super::sound_system::SoundSystem;
use super::window::{Window, WindowCommand};
use std::time::Duration;

/// Context that stores various update-related data.
pub struct Context<In> {
    delta: Duration,
    input: In,
    shall_stop: bool,
    window_commands: Vec<WindowCommand>,
    sound_system: Option<SoundSystem>,
}

impl<In> Context<In> {
    pub(super) fn new(delta: Duration, input: In, sound_system: Option<SoundSystem>) -> Self {
        let shall_stop = false;
        let window_commands = Vec::new();
        Self {
            delta,
            input,
            shall_stop,
            window_commands,
            sound_system,
        }
    }

    /// Get `Duration` of simulation step.
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Get reference to the `Input` structure.
    pub fn input(&self) -> &In {
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

    /// Get optional mutable reference to the `SoundSystem`.
    pub fn sound_system(&mut self) -> Option<&mut SoundSystem> {
        self.sound_system.as_mut()
    }

    pub(super) fn decompose(self) -> (Option<SoundSystem>, In, Vec<WindowCommand>) {
        (self.sound_system, self.input, self.window_commands)
    }
}
