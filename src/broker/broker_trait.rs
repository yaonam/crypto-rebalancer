use async_trait::async_trait;

#[async_trait]
pub trait Broker {
    async fn new(key: String, secret: String) -> Self;

    async fn connect();

    // Methods for strat
    fn get_total_value() -> f64;

    fn place_order();

    fn cancel_order();
}
