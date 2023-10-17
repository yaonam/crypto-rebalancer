use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum MarketData {
    OHLC(OHLCData),
    Trade(TradeData),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TickerData {
    pub ask_price: f64,
    pub ask_volume: f64,
    pub bid_price: f64,
    pub bid_volume: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OHLCData {
    pub pair: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub time: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TradeData {
    pub pair: String,
    pub price: f64,
    pub volume: f64,
    pub time: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderData {
    pub price: f64,
    pub volume: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderBookData {
    pub asks: Vec<OrderData>,
    pub bids: Vec<OrderData>,
}

pub struct LimitOrder {
    pub id: String,
    pub asset: String,
    pub volume: f64,
    pub price: f64,
    pub side: OrderSide,
    pub time: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum OrderStatus {
    Opened(OrderOpened),
    Filled(OrderFilled),
    Cancelled(OrderCancelled),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OrderSide {
    BUY,
    SELL,
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OrderSide::BUY => write!(f, "buy"),
            OrderSide::SELL => write!(f, "sell"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderOpened {
    pub id: String,
    pub asset: String,
    pub volume: f64,
    pub price: f64,
    pub status: String,
    pub side: OrderSide,
    pub time: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderFilled {
    pub id: String,
    // pub asset: String,
    // pub amount: f64,
    // pub price: f64,
    // pub status: String,
    // pub side: OrderSide,
    // pub time: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderCancelled {
    pub id: String,
    // pub asset: String,
    // pub amount: f64,
    // pub price: f64,
    // pub status: String,
    // pub side: OrderSide,
    // pub time: String,
}

pub fn deserialize_data(data: String) -> MarketData {
    match serde_json::from_str(&data) {
        Ok(T) => T,
        Err(e) => {
            println!("{}: {}", e, data);
            MarketData::OHLC(OHLCData {
                pair: String::new(),
                open: 0.0,
                high: 0.0,
                low: 0.0,
                close: 0.0,
                volume: 0.0,
                time: "0.0".to_string(),
            })
        }
    }
}

pub fn deserialize_order(data: String) -> OrderStatus {
    match serde_json::from_str(&data) {
        Ok(T) => T,
        Err(e) => {
            println!("{}: {}", e, data);
            OrderStatus::Opened(OrderOpened {
                id: "0.0".to_string(),
                asset: "0.0".to_string(),
                volume: 0.0,
                price: 0.0,
                status: "0.0".to_string(),
                side: OrderSide::BUY,
                time: "0.0".to_string(),
            })
        }
    }
}
