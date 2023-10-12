use async_trait::async_trait;

use crate::schema::LimitOrder;

#[async_trait]
pub trait Strategy {
    async fn on_data(&self);

    async fn on_order(&self);
}
