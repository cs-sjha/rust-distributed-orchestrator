use async_trait::async_trait;

#[async_trait]
pub trait Queue<T: Send>: Send + Sync {
    async fn enqueue(&self, item: T) -> Result<(), String>;
    async fn dequeue(&self) -> Result<Option<T>, String>;
}
