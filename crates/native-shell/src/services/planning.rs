use std::io::Result;

use crate::platform::notifications::{
    NotificationCenter, NotificationEvent, NotificationPreferences, NotificationTransport,
    ShellNotificationTransport,
};

#[derive(Debug, Clone)]
pub struct PlanningNotifications<Transport = ShellNotificationTransport> {
    project_name: String,
    preferences: NotificationPreferences,
    center: NotificationCenter<Transport>,
}

impl PlanningNotifications<ShellNotificationTransport> {
    pub fn new(project_name: String, preferences: NotificationPreferences) -> Self {
        Self {
            project_name,
            preferences,
            center: NotificationCenter::default(),
        }
    }
}

impl<Transport: NotificationTransport> PlanningNotifications<Transport> {
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

    pub fn notify_plan_completed(&self) -> Result<bool> {
        let title = format!("Plan completed: {}", self.project_name);
        let body = format!("Planning completed for {}.", self.project_name);
        self.center.notify(
            NotificationEvent::PlanningCompleted,
            &self.preferences,
            &title,
            &body,
        )
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
                .expect("planning test lock poisoned")
                .push((title.to_string(), body.to_string()));
            Ok(())
        }
    }

    #[test]
    fn notifies_when_planning_notifications_are_enabled() {
        let transport = TestTransport::new();
        let center = NotificationCenter::new(transport.clone());
        let service = PlanningNotifications::with_center(
            String::from("Refactor"),
            NotificationPreferences::default(),
            center,
        );
        let sent = service
            .notify_plan_completed()
            .expect("planning notification should succeed");
        assert!(sent);
        let deliveries = transport
            .deliveries
            .lock()
            .expect("planning test lock poisoned");
        assert_eq!(deliveries.len(), 1);
    }

    #[test]
    fn suppresses_when_planning_notifications_are_disabled() {
        let transport = TestTransport::new();
        let center = NotificationCenter::new(transport.clone());
        let preferences = NotificationPreferences {
            planning: false,
            completions: true,
            blocked: true,
            rate_limits: true,
        };
        let service = PlanningNotifications::with_center(String::from("Refactor"), preferences, center);
        let sent = service
            .notify_plan_completed()
            .expect("planning notification should succeed");
        assert!(!sent);
        assert!(
            transport
                .deliveries
                .lock()
                .expect("planning test lock poisoned")
                .is_empty()
        );
    }
}
