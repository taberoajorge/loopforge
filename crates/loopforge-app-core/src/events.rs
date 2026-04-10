use std::sync::{mpsc, Arc, Mutex};

use crate::{AtomizerEvent, LoopSessionEvent, PlanSessionEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Plan(PlanSessionEvent),
    Atomizer(AtomizerEvent),
    Loop(LoopSessionEvent),
}

#[derive(Debug, Clone, Default)]
pub struct EventFanout {
    subscribers: Arc<Mutex<Vec<mpsc::Sender<AppEvent>>>>,
}

impl EventFanout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> mpsc::Receiver<AppEvent> {
        let (sender, receiver) = mpsc::channel();
        self.subscribers
            .lock()
            .expect("event fanout lock poisoned")
            .push(sender);
        receiver
    }

    pub fn publish(&self, event: AppEvent) {
        let mut subscribers = self
            .subscribers
            .lock()
            .expect("event fanout lock poisoned");
        subscribers.retain(|subscriber| subscriber.send(event.clone()).is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fans_out_events_to_subscribers() {
        let fanout = EventFanout::new();
        let first = fanout.subscribe();
        let second = fanout.subscribe();
        let event = AppEvent::Loop(LoopSessionEvent::SessionEnded {
            project_id: String::from("project-1"),
        });
        fanout.publish(event.clone());
        assert_eq!(first.recv().unwrap(), event);
        assert_eq!(second.recv().unwrap(), event);
    }
}
