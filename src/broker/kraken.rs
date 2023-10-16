use super::broker_trait::{Broker, BrokerStatic};
use crate::schema::{deserialize_data, deserialize_order, LimitOrder, OrderBookData, OrderData};
use crate::strategy::Strategy;
use crate::websocket::connect;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use futures_util::stream::{SelectAll, SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use reqwest::header;
use serde_json::json;
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::sync::Arc;
use std::{str, time};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

const BASE_URL: &str = "https://api.kraken.com";
const WS_URL: &str = "wss://ws.kraken.com";
const WS_AUTH_URL: &str = "wss://ws-auth.kraken.com";

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct Kraken<T: Strategy> {
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

impl<T: Strategy> Kraken<T> {
    pub async fn new(key: String, secret: String, strat: T) -> Self {
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

        // Populate assets
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
impl<T: Strategy> Broker for Kraken<T> {
    async fn subscribe(
        &mut self,
        symbols: Vec<String>,
    ) -> (
        SplitSink<Socket, Message>,
        SplitStream<Socket>,
        SplitSink<Socket, Message>,
        SplitStream<Socket>,
        String,
    ) {
        let (mut pub_sink, mut pub_reader) = connect(WS_URL).await.unwrap();
        let (mut priv_sink, mut priv_reader) = connect(WS_AUTH_URL).await.unwrap();

        // Sub to tickers
        for symbol in symbols {
            let message = json!(
            {
                "event": "subscribe",
                "pair": [symbol],
                "subscription": {
                    "name": "ticker"
                }
            })
            .to_string();
            pub_sink.send(Message::Text(message)).await.unwrap();
        }

        // Get ws token
        let token = self.get_ws_token().await;
        // Sub to open orders
        let message = json!(
        {
            "event": "subscribe",
            "subscription": {
                "name": "openOrders",
                "token": token,
            }
        })
        .to_string();
        priv_sink.send(Message::Text(message)).await.unwrap();

        (pub_sink, pub_reader, priv_sink, priv_reader, token)
    }

    async fn start(
        &mut self,
        mut pub_reader: SplitStream<Socket>,
        mut priv_reader: SplitStream<Socket>,
    ) {
        loop {
            select! {
                pub_msg = pub_reader.next() => {
                    if let Some(Ok(Message::Text(data))) = pub_msg {
                        // TODO: convert kraken data to generic data
                        self.strat.on_data(deserialize_data(data)).await;
                    } else {
                        println!("Error: {:?}", pub_msg);
                    }
                },
                priv_msg = priv_reader.next() => {
                    if let Some(Ok(Message::Text(order))) = priv_msg {
                        self.strat.on_order(deserialize_order(order)).await;
                    } else {
                        println!("Error: {:?}", priv_msg);
                    }
                }
            }
        }
    }
}

pub struct KrakenStatic {}

impl KrakenStatic {
    pub fn new() -> Self {
        KrakenStatic {}
    }
}

#[async_trait]
impl BrokerStatic for KrakenStatic {
    async fn get_order_book(client: reqwest::Client, symbol: String) -> OrderBookData {
        const PATH: &str = "/0/public/Depth?pair=";

        let response = client
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

    async fn get_assets() -> f64 {
        0.0
    }

    async fn place_order(
        priv_sink: &mut SplitSink<Socket, Message>,
        order: LimitOrder,
        token: String,
    ) {
        let message = json!(
            {
                "event": "addOrder",
                "ordertype": "limit",
                "pair": order.asset,
                "price": order.price,
                "token": token,
                "type": order.side,
                "volume": order.volume.to_string(),
            }
        )
        .to_string();
        priv_sink.send(Message::Text(message)).await.unwrap();
    }

    async fn cancel_order(order: LimitOrder) {}
}
