use anyhow::Result;

pub trait App {
    /// Provide a user interface to the user, and handle user input by dispatching to the MessageHandler.
    /// Return Ok when the user chooses to exit, return Err only when encountering a fatal error
    /// It's annoying that we need to move self here because it means we can't call run on dyn App objects
    /// but i can't find a workaround for the way winit::EventLoop::run works. Doesn't matter anyway since
    /// theres only one implementation...
    fn run(self, controller: Box<dyn Controller>) -> Result<()>;
}

/// Definition for all possible control actions that can happen
/// Only return Err when encountering a fatal error
/// TODO make backend agnostic? ie. maybe we later implement for OSC, or socket connections or something
pub trait Controller {
    fn note_on(&mut self, note: wmidi::Note, velocity: wmidi::U7) -> Result<()>;
    fn note_off(&mut self, note: wmidi::Note) -> Result<()>;
    fn cc(&mut self, function: wmidi::ControlFunction) -> Result<()>;
    fn sustain_on(&mut self) -> Result<()>;
    fn sustain_off(&mut self) -> Result<()>;
}
