use std::io::Result;

use crate::platform::notifications::{
    NotificationCenter, NotificationEvent, NotificationPreferences, NotificationTransport,
    ShellNotificationTransport,
};

#[derive(Debug, Clone)]
pub struct LoopNotifications<Transport = ShellNotificationTransport> {
    project_name: String,
    preferences: NotificationPreferences,
    center: NotificationCenter<Transport>,
}

impl LoopNotifications<ShellNotificationTransport> {
    pub fn new(project_name: String, preferences: NotificationPreferences) -> Self {
        Self {
            project_name,
            preferences,
            center: NotificationCenter::default(),
        }
    }
}

impl<Transport: NotificationTransport> LoopNotifications<Transport> {
    pub fn with_center(
        project_name: String,
        preferences: NotificationPreferences,
        center: NotificationCenter<Transport>,
    ) -> Self {
        Self {
            project_name,
            preferences,
            center,
        }
    }

    pub fn set_preferences(&mut self, preferences: NotificationPreferences) {
        self.preferences = preferences;
    }

    pub fn notify_story_completed(&self, story_id: &str) -> Result<bool> {
        let title = format!("Story completed: {story_id}");
        let body = format!("{story_id} completed in {}.", self.project_name);
        self.center
            .notify(NotificationEvent::StoryCompleted, &self.preferences, &title, &body)
    }

    pub fn notify_story_blocked(&self, story_id: &str) -> Result<bool> {
        let title = format!("Story blocked: {story_id}");
        let body = format!("{story_id} blocked in {}.", self.project_name);
        self.center
            .notify(NotificationEvent::StoryBlocked, &self.preferences, &title, &body)
    }

    pub fn notify_rate_limited(&self) -> Result<bool> {
        let title = format!("Rate limited: {}", self.project_name);
        let body = format!("All agents reached rate limits in {}.", self.project_name);
        self.center
            .notify(NotificationEvent::RateLimited, &self.preferences, &title, &body)
    }

    pub fn notify_loop_completed(&self) -> Result<bool> {
        let title = format!("Loop completed: {}", self.project_name);
        let body = format!("Execution completed for {}.", self.project_name);
        self.center
            .notify(NotificationEvent::LoopCompleted, &self.preferences, &title, &body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct TestTransport {
        deliveries: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl TestTransport {
        fn new() -> Self {
            Self {
                deliveries: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl NotificationTransport for TestTransport {
        fn send(&self, title: &str, body: &str) -> Result<()> {
            self.deliveries
                .lock()
                .expect("loop test lock poisoned")
                .push((title.to_string(), body.to_string()));
            Ok(())
        }
    }

    #[test]
    fn emits_blocked_and_completion_notifications() {
        let transport = TestTransport::new();
        let center = NotificationCenter::new(transport.clone());
        let service = LoopNotifications::with_center(
            String::from("Refactor"),
            NotificationPreferences::default(),
            center,
        );
        let completion = service
            .notify_story_completed("S-041")
            .expect("completion notification should succeed");
        let blocked = service
            .notify_story_blocked("S-042")
            .expect("blocked notification should succeed");
        assert!(completion);
        assert!(blocked);
        let deliveries = transport
            .deliveries
            .lock()
            .expect("loop test lock poisoned");
        assert_eq!(deliveries.len(), 2);
    }

    #[test]
    fn suppresses_blocked_notifications_when_disabled() {
        let transport = TestTransport::new();
        let center = NotificationCenter::new(transport.clone());
        let preferences = NotificationPreferences {
            planning: true,
            completions: true,
            blocked: false,
            rate_limits: true,
        };
        let service = LoopNotifications::with_center(String::from("Refactor"), preferences, center);
        let sent = service
            .notify_story_blocked("S-041")
            .expect("blocked notification should succeed");
        assert!(!sent);
        assert!(
            transport
                .deliveries
                .lock()
                .expect("loop test lock poisoned")
                .is_empty()
        );
    }
}
