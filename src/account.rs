use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use hex;
use hmac::{Hmac, Mac};
use serde_urlencoded;
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::{str, time};

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
    secret_slice: [u8; 64],
}

impl Signer {
    pub fn new(key: String, secret: String) -> Self {
        Signer {
            key: key.clone(),
            secret: secret.clone(),
            secret_slice: general_purpose::STANDARD
                .decode(secret.as_str())
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
        }
    }

    pub fn sign(&self, url: String, data: Vec<(String, String)>) -> String {
        let nonce = time::UNIX_EPOCH.elapsed().unwrap().as_millis().to_string();
        // Add nonce to front of data
        let mut data = data;
        data.push(("nonce".to_string(), nonce));
        // let mut data = vec![("nonce".to_string(), nonce)];
        // data.extend(data);

        let mut hasher = Sha256::new();
        let payload = serde_urlencoded::to_string(data).unwrap();
        // let payload =
        //     b"nonce=1616492376594&ordertype=limit&pair=XBTUSD&price=37500&type=buy&volume=1.25";
        hasher.update(b"1616492376594");
        hasher.update(payload.as_bytes());
        let encoded_payload = &hasher.finalize();

        let content = [b"/0/private/AddOrder" as &[u8], encoded_payload].concat();
        let mut mac =
            Hmac::<Sha512>::new_from_slice(&self.secret_slice).expect("Couldn't create HMAC");
        mac.update(&content);
        let signature = mac.finalize().into_bytes();

        let sign = general_purpose::STANDARD.encode(signature);

        sign
    }
}
