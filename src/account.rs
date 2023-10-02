use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use hex;
use hmac::{Hmac, Mac};
use reqwest::{
    header::{self, HeaderValue},
    Method, Response, StatusCode,
};
use serde_urlencoded;
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::{str, time};

const BASE_URL: &str = "https://api.kraken.com";

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

/// Signer for Kraken API. Handles signing and sending requests.
pub struct Signer {
    key: String,
    secret: String,
    secret_slice: [u8; 64],
    client: reqwest::Client,
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
            client: reqwest::Client::new(),
        }
    }

    /// Returns a tuple of the signed data and the signature.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the request.
    /// * `data` - A vector of tuples of the form (key, value).
    pub fn sign(&self, url: &String, data: Vec<(&str, &str)>) -> (String, String) {
        let nonce = time::UNIX_EPOCH.elapsed().unwrap().as_millis().to_string();
        let mut data_stamped = data;
        data_stamped.push(("nonce", &nonce));
        let post_data = serde_urlencoded::to_string(&data_stamped).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(nonce.as_bytes());
        hasher.update(post_data.as_bytes());
        let encoded_payload = &hasher.finalize();

        let mut mac =
            Hmac::<Sha512>::new_from_slice(&self.secret_slice).expect("Couldn't create HMAC");
        mac.update(&url.as_bytes());
        mac.update(&encoded_payload);
        let signature = mac.finalize().into_bytes();

        let sign = general_purpose::STANDARD.encode(signature);

        (post_data, sign)
    }

    /// Returns the ws auth token.
    pub async fn get_ws_token(&self) -> String {
        let path = String::from("/0/private/GetWebSocketsToken");
        let url = format!("{}/0/private/GetWebSocketsToken", BASE_URL);
        let (post_data, sign) = self.sign(&path, vec![]);
        let mut headers = header::HeaderMap::new();
        headers.insert("API-Key", self.key.as_str().parse().unwrap());
        headers.insert("API-Sign", sign.as_str().parse().unwrap());

        let response = self
            .client
            .post(url.as_str())
            .headers(headers)
            .body(post_data)
            .send()
            .await
            .unwrap();

        let body = response.text().await.unwrap();

        body
    }
}
