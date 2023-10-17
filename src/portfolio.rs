
use hmac::{Mac};


use sha2::{Digest};
use std::collections::HashMap;




pub struct Portfolio {
    assets: HashMap<String, (f64, f64)>, // (amount, price)
}

impl Portfolio {
    pub fn new() -> Self {
        Portfolio {
            assets: HashMap::new(),
        }
    }

    /// Returns a tuple of the amount and price of the asset.
    pub fn get_asset(&self, asset: String) -> (f64, f64) {
        match self.assets.get(&asset) {
            Some((amount, price)) => (*amount, *price),
            None => (0.0, 0.0),
        }
    }

    // value/total - target. In percentage.
    pub fn get_asset_target_delta(&self, asset: String) -> f64 {
        let (amount, price) = self.get_asset(asset);
        let total_value = self.get_total_value();

        let target = 1.0 / self.assets.len() as f64;

        (amount * price / total_value - target) * 100.0
    }

    fn get_asset_allocation(&self, asset: String) -> f64 {
        let (amount, price) = self.get_asset(asset);
        amount * price / self.get_total_value()
    }

    fn get_total_value(&self) -> f64 {
        let mut total = 0.0;
        for (amount, price) in self.assets.values() {
            total += amount * price;
        }
        total
    }

    pub fn set_asset(&mut self, asset: String, amount: f64, price: f64) {
        self.assets.insert(asset, (amount, price));
    }
}
