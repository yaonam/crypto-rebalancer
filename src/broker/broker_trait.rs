use crate::schema::Data;
use crate::schema::LimitOrder;
use crate::schema::OrderBookData;
use async_trait::async_trait;

#[async_trait]
pub trait Broker {
    fn new(key: String, secret: String) -> Self;

    /// Adds another strategy. Will subscribe to appropriate channels and
    /// call the appropriate methods.
    async fn connect();

    /// Returns the order book for the given symbol.
    async fn get_order_book(&self, symbol: String) -> OrderBookData;

    fn get_total_value(&self) -> f64;

    fn place_order(&self, order: LimitOrder);

    fn cancel_order(&self, order: LimitOrder);
}
