use async_trait::async_trait;

use crate::schema::{MarketData, OrderStatus};

#[async_trait]
pub trait Strategy: Sync + Send {
    async fn on_data(&self, data: MarketData);

    async fn on_order(&self, order: OrderStatus);
}
