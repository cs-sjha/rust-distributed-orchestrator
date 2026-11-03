use async_trait::async_trait;
use domain::Job;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("job repository error: {0}")]
    Repository(String),
    #[error("job not found: {0}")]
    NotFound(Uuid),
}

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Job>, ApplicationError>;
}
