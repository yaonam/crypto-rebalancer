use crate::schema::LimitOrder;
use crate::schema::MarketData;
use crate::schema::OrderBookData;
use crate::strategy::Strategy;
use async_trait::async_trait;

#[async_trait]
pub trait Broker {
    async fn start(&mut self, symbols: Vec<String>);

    /// Returns the order book for the given symbol.
    async fn get_order_book(&self, symbol: String) -> OrderBookData;

    fn get_total_value(&self) -> f64;

    fn place_order(&self, order: LimitOrder);

    fn cancel_order(&self, order: LimitOrder);
}
