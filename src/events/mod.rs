pub mod handler;

use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

pub use handler::EventHandler;

/// Represents a raw event from the terminal or the system.
#[derive(Clone, Debug)]
pub enum Event {
    Key(KeyEvent),
    Resize,
    Tick,
    Error,
}

pub type EventHandlerSender = mpsc::UnboundedSender<Event>;
