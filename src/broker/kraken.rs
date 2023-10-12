use super::broker_trait::Broker;
use crate::schema::{LimitOrder, OrderBookData, OrderData};
use crate::strategy::Strategy;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use reqwest::header;
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::sync::Arc;
use std::{str, time};
use tokio::sync::Mutex;

const BASE_URL: &str = "https://api.kraken.com";

pub struct Kraken<T> {
    key: String,
    secret: String,
    secret_slice: [u8; 64],
    client: reqwest::Client,
    assets: HashMap<String, (f64, f64)>, // (amount, price)
    strat: T,
}

#[derive(serde::Deserialize)]
struct KrakenOrderBook {
    asks: Vec<(f64, f64, f64)>, // (price, volume, timestamp)
    bids: Vec<(f64, f64, f64)>,
}

impl<T> Kraken<T> {
    async fn new(key: String, secret: String, strat: T) -> Self {
        let mut kraken = Kraken {
            key: key.clone(),
            secret: secret.clone(),
            secret_slice: general_purpose::STANDARD
                .decode(secret.as_str())
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            client: reqwest::Client::new(),
            assets: HashMap::new(),
            strat,
        };

        let balances = kraken.get_account_balances().await;
        for (asset, balance) in balances.as_object().unwrap() {
            let amount = balance.as_str().unwrap().parse::<f64>().unwrap();
            let price = if asset == "ZUSD" { 1.0 } else { 0.0 };
            kraken.assets.insert(asset.clone(), (amount, price));
            println!("Found {}: {} @ {}", asset, amount, price)
        }

        println!("Initialized Kraken broker");
        kraken
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
    fn sign(&self, url: &str, data: Vec<(&str, &str)>) -> (String, String) {
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
    async fn get_ws_token(&self) -> String {
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

    async fn get_account_balances(&self) -> serde_json::Value {
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

#[async_trait]
impl<T> Broker for Kraken<T> {
    async fn connect(&mut self, symbol: String) {
        // TODO: subscribe to appropriate channels
    }

    async fn start(&mut self) {}

    async fn get_order_book(&self, symbol: String) -> OrderBookData {
        const PATH: &str = "/0/public/Depth?pair=";

        let response = self
            .client
            .get(format!("{}{}{}", BASE_URL, PATH, symbol).as_str())
            .send()
            .await
            .unwrap();

        let body = response.text().await.unwrap();

        let order_book: KrakenOrderBook = serde_json::from_str(&body).unwrap();

        let asks: Vec<OrderData> = order_book
            .asks
            .into_iter()
            .map(|(price, volume, _)| OrderData { price, volume })
            .collect();
        let bids = order_book
            .bids
            .into_iter()
            .map(|(price, volume, _)| OrderData { price, volume })
            .collect();
        OrderBookData { asks, bids }
    }

    fn get_total_value(&self) -> f64 {
        0.0
    }

    fn place_order(&self, order: LimitOrder) {}

    fn cancel_order(&self, order: LimitOrder) {}
}
