use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use reqwest::header;
use serde_urlencoded;
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::sync::Arc;
use std::{str, time};
use tokio::sync::Mutex;

const BASE_URL: &str = "https://api.kraken.com";

pub struct Portfolio {
    assets: HashMap<String, (f64, f64)>, // (amount, price)
    pub signer: Arc<Mutex<Signer>>,
}

impl Portfolio {
    pub async fn new(signer: Arc<Mutex<Signer>>) -> Self {
        println!("Initializing portfolio...");
        let balances = { signer.lock().await.get_account_balances().await };
        let mut assets = HashMap::new();
        for (asset, balance) in balances.as_object().unwrap() {
            let amount = balance.as_str().unwrap().parse::<f64>().unwrap();
            let price = if asset == "ZUSD" { 1.0 } else { 0.0 };
            println!("Found {}: {} @ {}", asset, amount, price);
            if amount == 0.0 {
                continue;
            }
            if let Some(stripped) = asset.strip_prefix("X") {
                println!("Inserting: {}", stripped);
                assets.insert(stripped.to_string(), (amount, price));
            } else {
                println!("Inserting: {}", asset);
                assets.insert(asset.clone(), (amount, price));
            }
        }
        Portfolio {
            assets: assets,
            signer: signer,
        }
    }

    /// Returns a tuple of the amount and price of the asset.
    pub fn get_pair(&self, pair: String) -> (f64, f64) {
        let asset = if let Some(stripped) = pair.strip_suffix("/USD") {
            stripped.to_string()
        } else {
            pair
        };
        match self.assets.get(&asset) {
            Some((amount, price)) => (*amount, *price),
            None => (0.0, 0.0),
        }
    }

    // value/total - target. In percentage.
    pub fn get_pair_target_delta(&self, pair: String) -> f64 {
        let (amount, price) = self.get_pair(pair);
        let total_value = self.get_total_value();

        let target = 1.0 / self.assets.len() as f64;

        if amount == 0.0 || price == 0.0 {
            return 0.0;
        }

        (target - amount * price / total_value) / target * 50.0
    }

    fn get_asset_allocation(&self, asset: String) -> f64 {
        let (amount, price) = self.get_pair(asset);
        amount * price / self.get_total_value()
    }

    fn get_total_value(&self) -> f64 {
        let mut total = 0.0;
        for (amount, price) in self.assets.values() {
            total += amount * price;
        }
        total
    }

    pub fn update_pair(&mut self, pair: String, order_vol: f64, order_price: f64) {
        let asset = if let Some(stripped) = pair.strip_suffix("/USD") {
            stripped.to_string()
        } else {
            pair
        };
        println!("Update asset: {}", asset);
        // Update token
        if let Some((amount, price)) = self.assets.get_mut(&asset) {
            *amount = *amount + order_vol;
            *price = order_price;
        } else {
            println!("Asset not found: {}", asset);
        }
        // Update USD
        if let Some((amount, _)) = self.assets.get_mut("ZUSD") {
            *amount = *amount - (order_vol * order_price);
        } else {
            println!("Asset not found: ZUSD");
        }
    }

    pub fn set_pair_price(&mut self, pair: String, new_price: f64) {
        if let Some(stripped) = pair.strip_suffix("/USD") {
            if let Some((_, price)) = self.assets.get_mut(stripped) {
                *price = new_price;
            } else {
                println!("Asset not found for {}", pair);
            };
        } else {
            println!("Not /USD pair: {}", pair);
        };
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
    pub async fn new(key: String, secret: String) -> Self {
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

    fn get_nonce(&self) -> String {
        time::UNIX_EPOCH.elapsed().unwrap().as_millis().to_string()
    }

    /// Returns a tuple of the signed data and the signature.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the request.
    /// * `data` - A vector of tuples of the form (key, value).
    pub fn sign(&self, url: &str, data: Vec<(&str, &str)>) -> (String, String) {
        let nonce = self.get_nonce();
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
        const PATH: &str = "/0/private/GetWebSocketsToken";

        let (post_data, sign) = self.sign(PATH, vec![]);
        let mut headers = header::HeaderMap::new();
        headers.insert("API-Key", self.key.as_str().parse().unwrap());
        headers.insert("API-Sign", sign.as_str().parse().unwrap());

        let response = self
            .client
            .post(format!("{}{}", BASE_URL, PATH).as_str())
            .headers(headers)
            .body(post_data)
            .send()
            .await
            .unwrap();

        let body = response.text().await.unwrap();

        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        json["result"]["token"].as_str().unwrap().to_string()
    }

    pub async fn get_account_balances(&self) -> serde_json::Value {
        const PATH: &str = "/0/private/Balance";

        let (post_data, sign) = self.sign(PATH, vec![]);
        let mut headers = header::HeaderMap::new();
        headers.insert("API-Key", self.key.as_str().parse().unwrap());
        headers.insert("API-Sign", sign.as_str().parse().unwrap());

        let response = self
            .client
            .post(format!("{}{}", BASE_URL, PATH).as_str())
            .headers(headers)
            .body(post_data)
            .send()
            .await
            .unwrap();

        let body = response.text().await.unwrap();

        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        json["result"].clone()
    }
}
