use crate::account::Portfolio;
use crate::messages::{OpenOrders, OrderData, PublicData, TickerData, WSPayload};
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

const BUFFER_SIZE: usize = 100; // Number of prices/spreads to keep in memory
const PRICE_RECORD_INTERVAL: u64 = 60; // seconds
const ORDER_SIZE_USD: f64 = 35.0;
const RISK_AVERSION: f64 = 5.0;
const FEE: f64 = 0.0016;
const UPDATE_PRICE_THRESHOLD: f64 = 0.0005;
const BASE_VOLATILITY: f64 = 0.0005;
const ORDER_CREATION_COOLDOWN: u64 = 5; // seconds

// Ratio for how much mid price updates
const PRICE_UPDATE_NUMERATOR: f64 = 3.0;
const PRICE_UPDATE_DENOMINATOR: f64 = 4.0;
const MIN_SPREAD: f64 = 4.0 * FEE * PRICE_UPDATE_NUMERATOR / PRICE_UPDATE_DENOMINATOR;

const DECIMALS: u8 = 99;

pub struct Market {
    // Constants
    pair: String,
    decimals: u8,

    // Market data
    last_price: f64,
    prices: VecDeque<f64>,
    prices_last_updated: u64,
    spreads: VecDeque<f64>,
    spreads_last_updated: u64,
    vol_24hr: f64,

    // Orders
    bid_orders: HashMap<String, OrderData>,
    ask_orders: HashMap<String, OrderData>,

    // Misc
    portfolio: Arc<Mutex<Portfolio>>,
    priv_sink: SplitSink<Socket, Message>,
    token: String, // Access token

    // To prevent multiple orders from being placed at the same time
    last_order_time: u64,
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
            decimals: DECIMALS,
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

            last_order_time: 0,
        }
    }

    pub async fn on_message(&mut self, message: String) {
        let deserialized: Result<WSPayload, serde_json::Error> = serde_json::from_str(&message);
        match deserialized {
            Ok(data) => match data {
                WSPayload::PublicMessage(pub_msg) => match pub_msg.data {
                    PublicData::Ticker(data) => self.on_ticker_data(data).await,
                    PublicData::OHLC(data) => {
                        self.record_price(data[6].as_str().unwrap().parse::<f64>().unwrap())
                            .await;
                    }
                    _ => {
                        println!("[{}] Unhandled public message: {:?}", self.pair, pub_msg);
                    }
                },
                WSPayload::OpenOrders(orders) => self.handle_order_update(orders).await,
                WSPayload::Heartbeat(_heartbeat) => {}
                _ => {
                    println!("[{}] Unhandled message: {:?}", self.pair, data);
                }
            },
            Err(e) => println!("[{}] Error: {}", self.pair, message),
            // Err(e) => println!("[{}] Error: {}, {}", self.pair, e, message),
        }
    }

    async fn handle_order_update(&mut self, orders: OpenOrders) {
        for order in orders.orders {
            for (order_id, order_data) in order {
                match order_data.status.as_str() {
                    "pending" | "open" => {
                        if order_data.descr.is_none() {
                            // println!("[{}] Order descr is None", self.pair);
                            continue;
                        }
                        if order_data.descr.as_ref().unwrap().pair != self.pair {
                            continue;
                        }
                        println!("[{}] Order {}: {}", self.pair, order_data.status, order_id);
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
                                    "[{}] Unhandled order type: {}",
                                    self.pair,
                                    order_data.descr.as_ref().unwrap()._type
                                );
                            }
                        }
                    }
                    "closed" => {
                        println!("[{}] Order filled: {}", self.pair, order_id);
                        if let Some(order) = self.bid_orders.remove(&order_id) {
                            self.on_order_filled(order).await;
                        } else if let Some(order) = self.ask_orders.remove(&order_id) {
                            self.on_order_filled(order).await;
                        }
                    }
                    "canceled" => {
                        if let Some(_) = self.bid_orders.remove(&order_id) {
                            println!("[{}] Bid cancelled: {}", self.pair, order_id)
                        } else if let Some(_) = self.ask_orders.remove(&order_id) {
                            println!("[{}] Ask cancelled: {}", self.pair, order_id)
                        }
                    }
                    _ => {
                        println!(
                            "[{}] Unhandled order status: {}",
                            self.pair, order_data.status
                        );
                    }
                }
            }
        }
    }

    async fn on_ticker_data(&mut self, data: TickerData) {
        let bid_price = data.b[0].as_str().unwrap().parse::<f64>().unwrap();
        let ask_price = data.a[0].as_str().unwrap().parse::<f64>().unwrap();
        self.record_spread(bid_price, ask_price);

        // Initialize
        let decimals = count_decimals(&bid_price.to_string());
        if self.get_last_price() == 0.0 {
            self.last_price = (bid_price + ask_price) / 2.0;
            self.decimals = decimals;
            self.record_traded_price().await;
        } else if decimals > self.decimals {
            self.decimals = decimals; // In case prev count had trailing zeros
        }

        self.vol_24hr = data.v[0].as_str().unwrap().parse::<f64>().unwrap();

        self.refresh_orders().await;
    }

    async fn on_order_filled(&mut self, order: OrderData) {
        println!("[{}] Order filled: {:?}", self.pair, order);

        let descr = &order.descr.unwrap();
        let order_price = descr.price.parse::<f64>().unwrap();
        let order_vol = order.vol.unwrap().parse::<f64>().unwrap();
        self.set_last_price(order_price);
        self.record_traded_price().await;

        {
            // Update portfolio balances
            let mut portfolio = self.portfolio.lock().await;
            if descr._type == "buy" {
                portfolio.update_pair(self.pair.clone(), order_vol, order_price)
            } else {
                portfolio.update_pair(self.pair.clone(), -order_vol, order_price)
            };
        }
    }

    async fn refresh_orders(&mut self) {
        let (reserve_price, optimal_spread) = self.get_ans_params().await;
        let bid_price = reserve_price * (1.0 - optimal_spread / 2.0);
        let ask_price = reserve_price * (1.0 + optimal_spread / 2.0);

        let bid_size = self.get_bid_size();
        let ask_size = self.get_ask_size();

        if !Market::similar_order_exists("buy", bid_price, &self.bid_orders)
            || !Market::similar_order_exists("sell", ask_price, &self.ask_orders)
        {
            if self.last_order_time + ORDER_CREATION_COOLDOWN
                > time::UNIX_EPOCH.elapsed().unwrap().as_secs()
            {
                println!("[{}] Order creation blocked by cooldown", self.pair);
                return;
            }
            self.last_order_time = time::UNIX_EPOCH.elapsed().unwrap().as_secs();

            self.cancel_orders().await;

            let buy_message = json!(
                {
                    "event": "addOrder",
                    "ordertype": "limit",
                    "pair": self.pair,
                    "price": self.round_price(bid_price),
                    "token": self.token,
                    "type": "buy",
                    "volume": bid_size.to_string(),
                }
            )
            .to_string();
            send(&mut self.priv_sink, &buy_message).await.unwrap();

            let sell_message = json!(
                {
                    "event": "addOrder",
                    "ordertype": "limit",
                    "pair": self.pair,
                    "price": self.round_price(ask_price),
                    "token": self.token,
                    "type": "sell",
                    "volume": ask_size.to_string(),
                }
            )
            .to_string();
            send(&mut self.priv_sink, &sell_message).await.unwrap();
        }
    }

    async fn cancel_orders(&mut self) {
        let keys: Vec<&str> = self
            .bid_orders
            .keys()
            .chain(self.ask_orders.keys())
            .map(|k| (*k).as_str())
            .collect();
        if !keys.is_empty() {
            let message = json!(
                {
                    "event": "cancelOrder",
                    "token": self.token,
                    "txid": keys
                }
            )
            .to_string();
            send(&mut self.priv_sink, &message).await.unwrap();
        }
    }

    async fn record_traded_price(&mut self) {
        let last_price = self.get_last_price();
        if last_price != 0.0 {
            let mut portfolio = self.portfolio.lock().await;
            portfolio.set_pair_price(self.pair.clone(), last_price);
        };
        self.prices_last_updated = 0;
        self.record_price(last_price).await;
    }

    /// Records self.prices if it has been PRICE_RECORD_INTERVAL seconds since the last recording.
    async fn record_price(&mut self, price: f64) {
        let now = time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        if now - self.prices_last_updated >= PRICE_RECORD_INTERVAL && price != 0.0 {
            self.prices.push_back(price);
            if self.prices.len() > BUFFER_SIZE {
                self.prices.pop_front();
            }
            self.prices_last_updated = now;

            // println!("[{}] Recorded new price: {}", self.pair, price);
            let (reserve_price, optimal_spread) = self.get_ans_params().await;
            println!(
                "[{}] Price scaling: {}, Optimal spread: {}",
                self.pair,
                reserve_price / self.get_last_price() - 1.0,
                optimal_spread
            );
        }
    }

    /// Records the spread if it has been PRICE_RECORD_INTERVAL seconds since the last recording.
    fn record_spread(&mut self, bid_price: f64, ask_price: f64) {
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

    async fn get_ans_params(&mut self) -> (f64, f64) {
        let volatility = self.get_volatility();
        let reserve_price = self.get_reserve_price(volatility).await;
        let optimal_spread = self.get_optimal_spread(reserve_price, volatility);

        (reserve_price, optimal_spread)
    }

    async fn get_reserve_price(&self, o: f64) -> f64 {
        let q = self.get_target_delta().await;
        let s = self.get_last_price();
        let y = RISK_AVERSION;
        // println!("[{}] Target delta: {}", self.pair, q);

        s * (1.0 + 3.0 * (q / q.abs().sqrt()) * y * o.powf(2.0))
    }

    fn get_optimal_spread(&mut self, reserve_price: f64, o: f64) -> f64 {
        let y = RISK_AVERSION;
        // let k = self.get_order_depth();

        let mut spread = y * o.powf(2.0) + (1.0 + y / 50.0).ln() / 2000.0;
        // let mut spread = y * o.powf(2.0) + (1.0 + y / k).ln() / 2000.0;

        if spread < MIN_SPREAD {
            spread = MIN_SPREAD;
        }

        // println!("[{}] Spread: {}, Volatility: {}", self.pair, spread, o);

        // Incorporate reserve price so doesn't cross mid price
        spread + 2.0 * (reserve_price / self.get_last_price() - 1.0).abs()
    }

    // Set last price to weighted avg of last price and new price
    fn set_last_price(&mut self, price: f64) {
        self.last_price = ((PRICE_UPDATE_DENOMINATOR - PRICE_UPDATE_NUMERATOR) * self.last_price
            + PRICE_UPDATE_NUMERATOR * price)
            / PRICE_UPDATE_DENOMINATOR;
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

    /// Returns the target delta for the asset. In percentage.
    async fn get_target_delta(&self) -> f64 {
        let portfolio = self.portfolio.lock().await;
        portfolio.get_pair_target_delta(self.pair.clone())
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

        variance.sqrt() / self.get_last_price() + BASE_VOLATILITY // Normalize by the last price
    }

    /// Returns an estimation of the liquidity.
    /// sqrt(vol_24hr*avg_spread)
    fn get_order_depth(&self) -> f64 {
        (self.vol_24hr * self.get_last_price()).ln()
    }

    fn round_price(&self, price: f64) -> String {
        let factor = 10.0_f64.powi(self.decimals as i32);
        ((price * factor).round() / factor).to_string()
    }

    fn similar_order_exists(_type: &str, price: f64, orders: &HashMap<String, OrderData>) -> bool {
        for (_, order) in orders {
            if order.descr.is_none() {
                continue;
            }
            let order_price = order.descr.as_ref().unwrap().price.parse::<f64>().unwrap();

            if (1.0 - order_price / price).abs() < UPDATE_PRICE_THRESHOLD {
                return true;
            }
        }
        false
    }
}

fn count_decimals(s: &str) -> u8 {
    // println!("Counting decimals for {}", s);
    if let Some(pos) = s.find('.') {
        s[pos + 1..].len() as u8
    } else {
        0
    }
}
