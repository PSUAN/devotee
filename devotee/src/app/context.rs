use super::input::Input;
use super::window::{Window, WindowCommand};
use std::time::Duration;

/// Context that stores various update-related data.
pub struct UpdateContext {
    delta: Duration,
    input: Input,
    shall_stop: bool,
    window_commands: Vec<WindowCommand>,
}

impl UpdateContext {
    pub(super) fn new(delta: Duration, input: Input) -> Self {
        let shall_stop = false;
        let window_commands = Vec::new();
        Self {
            delta,
            input,
            shall_stop,
            window_commands,
        }
    }

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

    pub(super) fn extract_window_commands(self) -> Vec<WindowCommand> {
        self.window_commands
    }
}
