mod kraken_msgs;
use self::kraken_msgs::{KrakenOpenOrders, KrakenPublicData, KrakenPublicMessage};
use super::broker_trait::{Broker, BrokerStatic};
use crate::schema::{
    deserialize_order, LimitOrder, MarketData, OHLCData, OrderBookData, OrderData, OrderOpened,
    OrderSide, OrderStatus, TradeData,
};
use crate::strategy::Strategy;
use crate::websocket::connect;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use reqwest::header;
use serde_json::json;
use sha2::{Digest, Sha256, Sha512};

use std::sync::Arc;
use std::{str, time};
use tokio::net::TcpStream;
use tokio::select;

use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const BASE_URL: &str = "https://api.kraken.com";
const WS_URL: &str = "wss://ws.kraken.com";
const WS_AUTH_URL: &str = "wss://ws-auth.kraken.com";

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct Kraken<T: Strategy> {
    key: String,
    secret: String,
    secret_slice: [u8; 64],
    client: reqwest::Client,
    pub_reader: Option<SplitStream<Socket>>,
    priv_reader: Option<SplitStream<Socket>>,
    strat: Option<Arc<T>>,
}

#[derive(serde::Deserialize)]
struct KrakenOrderBook {
    asks: Vec<(f64, f64, f64)>, // (price, volume, timestamp)
    bids: Vec<(f64, f64, f64)>,
}

impl<T: Strategy> Kraken<T> {
    pub async fn new(key: String, secret: String) -> Self {
        Kraken {
            key: key.clone(),
            secret: secret.clone(),
            secret_slice: general_purpose::STANDARD
                .decode(secret.as_str())
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            client: reqwest::Client::new(),
            pub_reader: Option::None,
            priv_reader: Option::None,
            strat: Option::None,
        }

        // Populate assets
        // let balances = kraken.get_account_balances().await;
        // for (asset, balance) in balances.as_object().unwrap() {
        //     let amount = balance.as_str().unwrap().parse::<f64>().unwrap();
        //     let price = if asset == "ZUSD" { 1.0 } else { 0.0 };
        //     kraken.assets.insert(asset.clone(), (amount, price));
        //     println!("Found {}: {} @ {}", asset, amount, price)
        // }

        // println!("Initialized Kraken broker");
        // kraken
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

    fn deserialize_data(s: &str) -> Result<Vec<MarketData>, serde_json::Error> {
        let msg = serde_json::from_str::<KrakenPublicMessage>(s)?;
        match msg.data {
            KrakenPublicData::OHLC(ohlc) => Ok(vec![MarketData::OHLC(OHLCData {
                pair: msg.pair,
                open: ohlc.open.parse().unwrap(),
                high: ohlc.high.parse().unwrap(),
                low: ohlc.low.parse().unwrap(),
                close: ohlc.close.parse().unwrap(),
                volume: ohlc.volume.parse().unwrap(),
                time: ohlc.time,
            })]),
            KrakenPublicData::Trade(trades) => Ok({
                let mut result = vec![];
                for trade in trades {
                    result.push(MarketData::Trade(TradeData {
                        pair: msg.pair.clone(),
                        price: trade.price.parse().unwrap(),
                        volume: trade.volume.parse().unwrap(),
                        time: trade.time,
                    }))
                }
                result
            }),
        }
    }

    fn deserialize_order(s: &str) -> Result<Vec<OrderStatus>, serde_json::Error> {
        let msg = serde_json::from_str::<KrakenOpenOrders>(s)?;
        let mut result: Vec<OrderStatus> = vec![];
        for order in msg.orders {
            for (order_id, order_data) in order {
                match order_data.status.as_str() {
                    "open" => {
                        let descr = order_data.descr.unwrap();
                        result.push(OrderStatus::Opened(OrderOpened {
                            id: order_id,
                            pair: descr.pair,
                            volume: order_data.vol.unwrap_or_default().parse().unwrap(),
                            price: descr.price.parse().unwrap(),
                            status: "open".to_string(),
                            side: OrderSide::BUY,
                            time: String::new(),
                        }))
                    }
                    _ => {}
                }
            }
        }
        Ok(result)
    }
}

#[async_trait]
impl<T: Strategy + 'static> Broker<T> for Kraken<T> {
    async fn connect(
        &mut self,
        symbols: Vec<&str>,
    ) -> (
        SplitSink<Socket, Message>,
        SplitSink<Socket, Message>,
        String,
    ) {
        let (mut pub_sink, pub_reader) = connect(WS_URL).await.unwrap();
        let (mut priv_sink, priv_reader) = connect(WS_AUTH_URL).await.unwrap();

        self.pub_reader = Some(pub_reader);
        self.priv_reader = Some(priv_reader);

        // Sub to ohlc
        let message = json!(
        {
            "event": "subscribe",
            "pair": symbols,
            "subscription": {
                "interval": 5,
                "name": "ohlc"
            }
        })
        .to_string();
        pub_sink.send(Message::Text(message)).await.unwrap();

        // Sub to trades
        let message = json!(
        {
            "event": "subscribe",
            "pair": symbols,
            "subscription": {
                "name": "trade"
            }
        })
        .to_string();
        pub_sink.send(Message::Text(message)).await.unwrap();

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

        // Get account balances
        let balances = self.get_account_balances().await;

        (pub_sink, priv_sink, token)
    }

    fn set_strat(&mut self, strat: T) {
        self.strat = Some(Arc::new(strat));
    }

    async fn start(&mut self) {
        let strat = self.strat.as_ref().unwrap().clone();
        loop {
            select! {
                pub_msg = self.pub_reader.as_mut().unwrap().next() => {
                    if let Some(Ok(Message::Text(data))) = pub_msg {
                        if data == r#"{"event":"heartbeat"}"# {
                            continue
                        }
                        if let Ok(deserialized_data) = Kraken::<T>::deserialize_data(&data) {
                            for deserialized in deserialized_data{
                                let strat_clone = strat.clone();
                                tokio::spawn(async move {strat_clone.on_data(deserialized).await});
                            }
                        } else {
                            println!("Error deserializing public: {}", data);
                        }
                    } else {
                        println!("Error: {:?}", pub_msg);
                    }
                },
                priv_msg = self.priv_reader.as_mut().unwrap().next() => {
                    if let Some(Ok(Message::Text(order))) = priv_msg {
                        if order == r#"{"event":"heartbeat"}"# {
                            continue
                        }
                        if let Ok(deserialized_orders) = Kraken::<T>::deserialize_order(&order) {
                            for deserialized in deserialized_orders {
                                let strat_clone = strat.clone();
                                tokio::spawn(async move {strat_clone.on_order(deserialized).await});
                            }
                        }
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
    async fn get_mid_price(client: reqwest::Client, symbol: String) -> f64 {
        const PATH: &str = "/0/public/Depth?count=1&pair=";

        let response = client
            .get(format!("{}{}{}", BASE_URL, PATH, symbol).as_str())
            .send()
            .await
            .unwrap();

        let body = response.text().await.unwrap();

        let order_book: KrakenOrderBook = serde_json::from_str(&body).unwrap();

        // let asks: Vec<OrderData> = order_book
        //     .asks
        //     .into_iter()
        //     .map(|(price, volume, _)| OrderData { price, volume })
        //     .collect();
        // let bids = order_book
        //     .bids
        //     .into_iter()
        //     .map(|(price, volume, _)| OrderData { price, volume })
        //     .collect();
        // OrderBookData { asks, bids }

        let ask_price = order_book.asks[0].0;
        let bid_price = order_book.bids[0].0;

        ask_price + bid_price / 2.0
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
                "pair": order.pair,
                "price": order.price,
                "token": token,
                "type": order.side,
                "volume": order.volume.to_string(),
            }
        )
        .to_string();
        priv_sink.send(Message::Text(message)).await.unwrap();
    }

    async fn cancel_order(_order: LimitOrder) {}
}
