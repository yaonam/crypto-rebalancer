use crate::account::Portfolio;
use crate::messages::{OpenOrders, OrderData, PublicData, PublicMessage, TickerData, WSPayload};
use crate::websocket::send;
use futures_util::stream::SplitSink;
use serde_json;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const BUFFER_SIZE: usize = 100;
const PRICE_RECORD_INTERVAL: u64 = 60; // seconds
const ORDER_SIZE_USD: f64 = 0.10;
const RISK_AVERSION: f64 = 0.5;

pub struct Market {
    pair: String,
    last_price: f64,
    prices: VecDeque<f64>,
    prices_last_updated: u64,
    spreads: VecDeque<f64>,
    spreads_last_updated: u64,
    vol_24hr: f64,

    bid_orders: HashMap<String, OrderData>,
    ask_orders: HashMap<String, OrderData>,

    portfolio: Arc<Mutex<Portfolio>>,
    priv_sink: SplitSink<Socket, Message>,
    token: String, // Access token
}

impl Market {
    pub fn new(
        pair: String,
        portfolio: Arc<Mutex<Portfolio>>,
        priv_sink: SplitSink<Socket, Message>,
        token: String,
    ) -> Self {
        Market {
            pair,
            last_price: 0.0,
            prices: VecDeque::with_capacity(BUFFER_SIZE),
            prices_last_updated: 0,
            spreads: VecDeque::with_capacity(BUFFER_SIZE),
            spreads_last_updated: 0,
            vol_24hr: 0.0,

            bid_orders: HashMap::new(),
            ask_orders: HashMap::new(),

            portfolio,
            priv_sink,
            token,
        }
    }

    pub async fn on_message(&mut self, message: String) {
        let deserialized: Result<WSPayload, serde_json::Error> = serde_json::from_str(&message);
        match deserialized {
            Ok(data) => match data {
                WSPayload::PublicMessage(pub_msg) => {
                    println!("{:?}", pub_msg);
                    match pub_msg.data {
                        PublicData::Ticker(data) => self.on_data(data).await,
                        _ => {}
                    }
                }
                WSPayload::OpenOrders(orders) => self.handle_order_update(orders).await,
                _ => {}
            },
            Err(e) => println!("Error: {}", e),
        }
    }

    async fn handle_order_update(&mut self, orders: OpenOrders) {
        for order in orders.orders {
            for (order_id, order_data) in order {
                match order_data.status.as_str() {
                    "open" => {
                        if order_data.descr.as_ref().unwrap().pair != self.pair {
                            continue;
                        }
                        match order_data.descr.as_ref().unwrap()._type.as_str() {
                            "buy" => {
                                if !self.bid_orders.contains_key(&order_id) {
                                    self.bid_orders.insert(order_id.clone(), order_data);
                                }
                            }
                            "sell" => {
                                if !self.ask_orders.contains_key(&order_id) {
                                    self.ask_orders.insert(order_id.clone(), order_data);
                                }
                            }
                            _ => {
                                println!(
                                    "Unhandled order type: {}",
                                    order_data.descr.as_ref().unwrap()._type
                                );
                            }
                        }
                    }
                    "closed" => {
                        if let Some(order) = self.bid_orders.remove(&order_id) {
                            self.on_order_filled(order).await;
                        } else if let Some(order) = self.ask_orders.remove(&order_id) {
                            self.on_order_filled(order).await;
                        }
                    }
                    "canceled" => {
                        self.bid_orders.remove(&order_id);
                        self.ask_orders.remove(&order_id);
                    }
                    _ => {
                        println!("Unhandled order status: {}", order_data.status);
                    }
                }
            }
        }
    }

    async fn on_data(&mut self, data: TickerData) {
        let bid_price = data.b[0].as_str().unwrap().parse::<f64>().unwrap();
        let ask_price = data.a[0].as_str().unwrap().parse::<f64>().unwrap();

        if self.last_price == 0.0 {
            self.last_price = (bid_price + ask_price) / 2.0;
        }
        self.record_price();

        self.vol_24hr = data.v[0].as_str().unwrap().parse::<f64>().unwrap();
        self.record_spread(bid_price, ask_price);

        let reserve_price = self.get_reserve_price().await;
        let bid_price = reserve_price * (1.0 - self.get_optimal_spread());
        let ask_price = reserve_price * (1.0 + self.get_optimal_spread());

        println!("Last price: {}", self.last_price);
        println!("Reserve price: {}", reserve_price);
        println!("Optimal spread: {}", self.get_optimal_spread());
        println!("Bid price: {}", bid_price);
        println!("Ask price: {}", ask_price);
    }

    async fn on_order_filled(&mut self, order: OrderData) {
        let descr = &order.descr.unwrap();
        let price = descr.price.parse::<f64>().unwrap();
        self.last_price = price;
        self.record_price();

        {
            // Update portfolio balances
            let mut portfolio = self.portfolio.lock().await;
            let (amount, _) = portfolio.get_asset(self.pair.clone());
            let new_amount: f64 = if descr._type == "buy" {
                amount + order.vol.unwrap().parse::<f64>().unwrap()
            } else {
                amount - order.vol.unwrap().parse::<f64>().unwrap()
            };
            portfolio.set_asset(self.pair.clone(), new_amount, price);
        }

        self.cancel_orders().await;
        self.create_orders().await;
    }

    async fn create_orders(&mut self) {
        let reserve_price = self.get_reserve_price().await;
        let bid_price = reserve_price * (1.0 - self.get_optimal_spread());
        let ask_price = reserve_price * (1.0 + self.get_optimal_spread());

        let bid_size = self.get_bid_size();
        let ask_size = self.get_ask_size();

        let message = json!(
            {
                "event": "addOrder",
                "ordertype": "limit",
                "pair": self.pair,
                "price": bid_price,
                "token": self.token,
                "type": "buy",
                "volume": bid_size,
            }
        )
        .to_string();
        send(&mut self.priv_sink, &message).await.unwrap();

        let message = json!(
            {
                "event": "addOrder",
                "ordertype": "limit",
                "pair": self.pair,
                "price": ask_price,
                "token": self.token,
                "type": "sell",
                "volume": ask_size,
            }
        )
        .to_string();
        send(&mut self.priv_sink, &message).await.unwrap();
    }

    async fn cancel_orders(&mut self) {
        let keys: Vec<&str> = self
            .bid_orders
            .keys()
            .chain(self.ask_orders.keys())
            .map(|k| (*k).as_str())
            .collect();
        let message = json!(
            {
                "event": "cancelOrder",
                "token": self.token,
                "txid": keys,
            }
        )
        .to_string();
        send(&mut self.priv_sink, &message).await.unwrap();
    }

    /// Records self.last_price if it has been PRICE_RECORD_INTERVAL seconds since the last recording.
    fn record_price(&mut self) {
        println!("Recording price...");
        let now = time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        if now - self.prices_last_updated >= PRICE_RECORD_INTERVAL {
            self.prices.push_back(self.last_price);
            if self.prices.len() > BUFFER_SIZE {
                self.prices.pop_front();
            }
            self.prices_last_updated = now;
        }
    }

    /// Records the spread if it has been PRICE_RECORD_INTERVAL seconds since the last recording.
    fn record_spread(&mut self, bid_price: f64, ask_price: f64) {
        println!("Recording spread...");
        let now = time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        if now - self.spreads_last_updated >= PRICE_RECORD_INTERVAL {
            let spread = 2.0 * (ask_price - bid_price) / (ask_price + bid_price);
            self.spreads.push_back(spread);
            if self.spreads.len() > BUFFER_SIZE {
                self.spreads.pop_front();
            }
            self.spreads_last_updated = now;
        }
    }

    async fn get_reserve_price(&self) -> f64 {
        let q = self.get_target_delta().await;
        let s = self.get_last_price();
        let y = RISK_AVERSION;
        let o = self.get_volatility();

        s + q * y * o.powf(2.0)
    }

    fn get_optimal_spread(&mut self) -> f64 {
        let y = RISK_AVERSION;
        let o = self.get_volatility();
        let k = self.get_order_depth();

        y * o.powf(2.0) + (1.0 + y / k).ln()
    }

    fn get_last_price(&self) -> f64 {
        self.last_price
    }

    fn get_bid_size(&mut self) -> f64 {
        ORDER_SIZE_USD / self.get_last_price()
    }

    fn get_ask_size(&mut self) -> f64 {
        ORDER_SIZE_USD / self.get_last_price()
    }

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

    fn get_order_depth(&self) -> f64 {
        1.0
    }
}
