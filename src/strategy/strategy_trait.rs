use async_trait::async_trait;

use crate::schema::LimitOrder;

#[async_trait]
pub trait Strategy {
    fn new();

    async fn on_data();

    async fn on_order();
}
