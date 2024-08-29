use std::{thread, time::Duration};

use winston::transports::Transport;

pub struct DelayedTransport {
    delay: Duration,
    done_sender: crossbeam_channel::Sender<()>,
}

impl DelayedTransport {
    pub fn new(delay: Duration, done_sender: crossbeam_channel::Sender<()>) -> Self {
        DelayedTransport { delay, done_sender }
    }
}

impl Transport for DelayedTransport {
    fn log(&self, message: &str, level: &str) {
        let delay: Duration = self.delay;
        let done_sender = self.done_sender.clone();
        let message = message.to_string();
        let level = level.to_string();

        thread::spawn(move || {
            thread::sleep(delay);
            println!("Delayed log: {} - {}", level, message);
            done_sender
                .send(())
                .expect("Failed to send completion signal");
        });
    }

    fn get_level(&self) -> Option<&String> {
        None
    }

    fn get_format(&self) -> Option<&crate::format::Format> {
        None
    }
}
