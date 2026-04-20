use std::sync::{mpsc, Arc, Mutex};

use crate::{atomizer::AtomizerProgress, AtomizerEvent, LoopSessionEvent, PlanSessionEvent};

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
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(sender);
        receiver
    }

    pub fn publish(&self, event: AppEvent) {
        let mut subscribers = self
            .subscribers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        subscribers.retain(|subscriber| subscriber.send(event.clone()).is_ok());
    }
}

pub fn atomizer_progress_payloads(events: &[AtomizerEvent]) -> Vec<AtomizerProgress> {
    events
        .iter()
        .map(AtomizerEvent::as_progress_payload)
        .collect::<Vec<_>>()
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

    #[test]
    fn converts_atomizer_events_into_progress_payloads() {
        let events = vec![AtomizerEvent::StageStarted {
            project_id: String::from("project-1"),
            stage: crate::AtomizerStage::CollectPlan,
        }];
        let payloads = atomizer_progress_payloads(&events);
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].stage, 1);
        assert_eq!(payloads[0].stage_name, "summarize");
    }
}
