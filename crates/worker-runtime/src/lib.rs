use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub concurrency: usize,
    pub poll_interval: Duration,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            concurrency: 4,
            poll_interval: Duration::from_secs(1),
        }
    }
}
