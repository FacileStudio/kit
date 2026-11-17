/// Where progress goes. The CLI prints it; a GUI shows it. Every method defaults to silence.
pub trait Reporter: Sync {
    /// A step has started.
    fn step(&self, _message: &str) {}
    /// Something finished well.
    fn success(&self, _message: &str) {}
    /// Something went wrong but the run goes on.
    fn warn(&self, _message: &str) {}
    /// A command or path the user may want to act on.
    fn hint(&self, _message: &str) {}
}

/// A reporter that drops every message.
#[derive(Debug, Default, Clone, Copy)]
pub struct Quiet;

impl Reporter for Quiet {}
