#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    NoUpdate,
    UpdateAvailable {
        version: String,
        notes: Option<String>,
    },
    Failed(String),
}

#[derive(Clone, Debug)]
pub enum UpdateCheckOutcome {
    NoUpdate,
    UpdateAvailable {
        version: String,
        notes: Option<String>,
    },
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct NativeUpdater {
    manual_outcome: UpdateCheckOutcome,
    background_outcome: Option<UpdateCheckOutcome>,
}

impl NativeUpdater {
    pub fn new(manual_outcome: UpdateCheckOutcome) -> Self {
        Self {
            manual_outcome,
            background_outcome: None,
        }
    }

    pub fn with_background_outcome(mut self, background_outcome: UpdateCheckOutcome) -> Self {
        self.background_outcome = Some(background_outcome);
        self
    }

    pub fn manual_check(&self) -> UpdateStatus {
        Self::map_outcome(self.manual_outcome.clone())
    }

    pub fn background_check(&self) -> UpdateStatus {
        let resolved_outcome = self
            .background_outcome
            .clone()
            .unwrap_or_else(|| self.manual_outcome.clone());
        Self::map_outcome(resolved_outcome)
    }

    fn map_outcome(outcome: UpdateCheckOutcome) -> UpdateStatus {
        match outcome {
            UpdateCheckOutcome::NoUpdate => UpdateStatus::NoUpdate,
            UpdateCheckOutcome::UpdateAvailable { version, notes } => {
                UpdateStatus::UpdateAvailable { version, notes }
            }
            UpdateCheckOutcome::Failed(message) => UpdateStatus::Failed(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NativeUpdater, UpdateCheckOutcome, UpdateStatus};

    #[test]
    fn manual_check_reports_no_update() {
        let updater = NativeUpdater::new(UpdateCheckOutcome::NoUpdate);
        assert_eq!(updater.manual_check(), UpdateStatus::NoUpdate);
    }

    #[test]
    fn manual_check_reports_update_available() {
        let updater = NativeUpdater::new(UpdateCheckOutcome::UpdateAvailable {
            version: "2.1.0".to_string(),
            notes: Some("Includes reliability fixes".to_string()),
        });
        assert_eq!(
            updater.manual_check(),
            UpdateStatus::UpdateAvailable {
                version: "2.1.0".to_string(),
                notes: Some("Includes reliability fixes".to_string()),
            }
        );
    }

    #[test]
    fn background_check_uses_background_outcome_when_configured() {
        let updater = NativeUpdater::new(UpdateCheckOutcome::NoUpdate).with_background_outcome(
            UpdateCheckOutcome::UpdateAvailable {
                version: "2.2.0".to_string(),
                notes: None,
            },
        );
        assert_eq!(
            updater.background_check(),
            UpdateStatus::UpdateAvailable {
                version: "2.2.0".to_string(),
                notes: None
            }
        );
    }
}
