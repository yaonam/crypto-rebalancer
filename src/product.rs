use crate::account::Portfolio;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

const BUFFER_SIZE: usize = 100;

pub struct Market {
    pair: String,
    last_price: f64,
    prices: VecDeque<f64>,
    bid_ticket: String,
    ask_ticket: String,
    risk_aversion: f64,
    portfolio: Arc<Mutex<Portfolio>>,
}

impl Market {
    pub fn new(pair: String, portfolio: Arc<Mutex<Portfolio>>) -> Self {
        Market {
            pair: pair,
            last_price: 0.0,
            prices: VecDeque::with_capacity(BUFFER_SIZE),
            bid_ticket: String::new(),
            ask_ticket: String::new(),
            risk_aversion: 0.0,
            portfolio: portfolio,
        }
    }

    pub async fn on_message(&self, message: String) {
        println!("{}", message);
    }

    fn on_data() {}

    fn on_order_filled() {}

    fn record_price() {}

    async fn get_reserve_price(&self) -> f64 {
        let q = self.get_target_delta().await;
        let s = self.get_last_price();
        let y = self.risk_aversion;
        let o = self.get_volatility();

        s + q * y * o.powf(2.0)
    }

    fn get_optimal_spread() {}

    fn get_last_price(&self) -> f64 {
        self.last_price
    }

    fn set_last_price() {}

    fn get_allocation() {}

    fn set_allocation() {}

    fn get_bid_size() {}

    fn get_ask_size() {}

    async fn get_target_delta(&self) -> f64 {
        let delta;
        {
            let portfolio = self.portfolio.lock().await;
            delta = portfolio.get_asset_target_delta(self.pair.clone());
        }
        delta
    }

    // Returns the standard deviation of the last 100 prices
    fn get_volatility(&self) -> f64 {
        let mut sum = 0.0;
        let mut count = 0.0;
        let mut variance = 0.0;

        for price in self.prices.iter() {
            sum += price;
            count += 1.0;
        }

        let mean = sum / count;

        for price in self.prices.iter() {
            variance += (price - mean).powf(2.0);
        }

        variance /= count;

        variance.sqrt()
    }

    fn get_order_depth() {}
}
