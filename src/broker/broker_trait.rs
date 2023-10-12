use crate::schema::Data;
use crate::schema::LimitOrder;
use crate::schema::OrderBookData;
use crate::strategy::Strategy;
use async_trait::async_trait;

#[async_trait]
pub trait Broker<T: Strategy> {
    /// Adds another strategy. Will subscribe to appropriate channels and
    /// call the appropriate methods.
    async fn connect(&mut self, strat: T);

    async fn start(&mut self);

    /// Returns the order book for the given symbol.
    async fn get_order_book(&self, symbol: String) -> OrderBookData;

    fn get_total_value(&self) -> f64;

    fn place_order(&self, order: LimitOrder);

    fn cancel_order(&self, order: LimitOrder);
}
