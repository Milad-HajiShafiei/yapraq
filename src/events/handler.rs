use crate::events::{Event, EventHandlerSender};
use crossterm::event::EventStream;
use futures::{FutureExt, StreamExt};
use std::time::Duration;
use tokio::time::interval;

/// A background task that listens for crossterm events and periodic ticks.
pub struct EventHandler;

impl EventHandler {
    pub fn start(tx: EventHandlerSender) {
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = interval(Duration::from_millis(250));

            loop {
                let tick_delay = tick_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = tick_delay => {
                        if tx.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    Some(result) = crossterm_event => {
                        match result {
                            Ok(evt) => {
                                let event = match evt {
                                    crossterm::event::Event::Key(key) => Some(Event::Key(key)),
                                    crossterm::event::Event::Resize(_, _) => Some(Event::Resize),
                                    crossterm::event::Event::Mouse(_) => None,
                                    _ => None,
                                };
                                if let Some(event) = event {
                                    if tx.send(event).is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(_) => {
                                let _ = tx.send(Event::Error);
                                break;
                            }
                        }
                    }
                    else => break,
                }
            }
        });
    }
}
