use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub kind: String,
    pub status: JobStatus,
    pub attempts: u32,
    pub max_attempts: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum JobTransitionError {
    #[error("cannot transition from {from:?} to {to:?}")]
    InvalidTransition { from: JobStatus, to: JobStatus },
}

impl JobStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        use JobStatus::*;
        matches!(
            (self, next),
            (Pending, Scheduled | Running | Cancelled)
                | (Scheduled, Running | Cancelled)
                | (Running, Completed | Failed)
                | (Failed, Pending)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_job_can_be_scheduled_or_cancelled() {
        assert!(JobStatus::Pending.can_transition_to(JobStatus::Scheduled));
        assert!(JobStatus::Pending.can_transition_to(JobStatus::Cancelled));
        assert!(!JobStatus::Pending.can_transition_to(JobStatus::Completed));
    }

    #[test]
    fn completed_job_is_terminal() {
        assert!(!JobStatus::Completed.can_transition_to(JobStatus::Pending));
        assert!(!JobStatus::Completed.can_transition_to(JobStatus::Failed));
    }
}
