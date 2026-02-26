use crossterm::event::{self, Event as CEvent, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse,
    Resize,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler with the specified tick rate.
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("Failed to poll new events") {
                        match event::read().expect("Unable to read event") {
                            CEvent::Key(e) => sender.send(Event::Key(e)),
                            CEvent::Mouse(_) => sender.send(Event::Mouse),
                            CEvent::Resize(_, _) => sender.send(Event::Resize),
                            CEvent::FocusGained => Ok(()),
                            CEvent::FocusLost => Ok(()),
                            CEvent::Paste(_) => unimplemented!(),
                        }
                        .expect("Failed to send terminal event")
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender.send(Event::Tick).expect("Failed to send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }
}
