use async_trait::async_trait;

#[async_trait]
pub trait Strategy: Send + Sync {
    async fn on_data(&self);

    async fn on_order(&self);
}
