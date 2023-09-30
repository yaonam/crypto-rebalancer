use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use hex;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::str;

pub struct Portfolio {
    assets: HashMap<String, (f64, f64)>, // (amount, price)
}

impl Portfolio {
    pub fn new() -> Self {
        Portfolio {
            assets: HashMap::new(),
        }
    }

    fn get_asset(&self, asset: String) -> (f64, f64) {
        match self.assets.get(&asset) {
            Some((amount, price)) => (*amount, *price),
            None => (0.0, 0.0),
        }
    }

    // value/total - target. In units of base asset (USD).
    pub fn get_asset_target_delta(&self, asset: String) -> f64 {
        let (amount, price) = self.get_asset(asset);
        let total_value = self.get_total_value();

        let target = 1.0 / self.assets.len() as f64;

        amount * price / total_value - target
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

pub struct Signer {
    key: String,
    secret: String,
}

impl Signer {
    pub fn new(key: String, secret: String) -> Self {
        Signer {
            key: key,
            secret: secret,
        }
    }

    pub fn sign(&self, url: String, data: String) -> String {
        let mut hasher = Sha256::new();
        let payload =
            b"nonce=1616492376594&ordertype=limit&pair=XBTUSD&price=37500&type=buy&volume=1.25";
        hasher.update(b"1616492376594");
        hasher.update(payload);
        let encoded_payload = &hasher.finalize();

        println!("encoded_payload: {:?}", encoded_payload);

        let key_base64 = "kQH5HW/8p1uGOVjbgWA7FunAmGO8lsSUXNsu3eow76sz84Q18fWxnyRzBHCd3pd5nE9qa99HAZtuZuj6F1huXg==";
        let key_vec = general_purpose::STANDARD.decode(key_base64).unwrap();
        let key_bytes = key_vec.as_slice();
        let content = [b"/0/private/AddOrder" as &[u8], encoded_payload].concat();
        let mut mac = Hmac::<Sha512>::new_from_slice(key_bytes).expect("Couldn't create HMAC");
        mac.update(&content);
        let signature = mac.finalize().into_bytes();

        let sign = general_purpose::STANDARD.encode(signature);

        sign
    }
}
