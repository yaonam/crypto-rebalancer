pub struct Product {
    pair: String,
    mid_price: f64,
    prices: Vec<f64>,
    bid_ticket: f64,
    ask_ticket: f64,
}

impl Product {
    pub fn new(pair: String) -> Self {
        Product {
            pair: pair,
            mid_price: 0.0,
            prices: Vec::new(),
            bid_ticket: 0.0,
            ask_ticket: 0.0,
        }
    }

    pub fn on_message(&self, message: String) {
        println!("{}", message);
    }

    fn on_data() {}

    fn on_order_filled() {}

    fn record_price() {}

    fn get_reserve_price() {}

    fn get_optimal_spread() {}

    fn get_last_price() {}

    fn set_last_price() {}

    fn get_allocation() {}

    fn set_allocation() {}

    fn get_bid_size() {}

    fn get_ask_size() {}

    fn get_dist_to_target() {}

    fn get_volatility() {}

    fn get_order_depth() {}
}
