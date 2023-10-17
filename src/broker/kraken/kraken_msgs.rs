use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct KrakenTickerData {
    pub a: Vec<serde_json::Value>,
    pub b: Vec<serde_json::Value>,
    // c: Vec<serde_json::Value>,
    pub v: Vec<serde_json::Value>,
    // p: Vec<serde_json::Value>,
    // t: Vec<serde_json::Value>,
    // l: Vec<serde_json::Value>,
    // h: Vec<serde_json::Value>,
    // o: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KrakenOHLCData {
    pub time: String,
    pub etime: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub vwap: String,
    pub volume: String,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KrakenTradeData {
    pub price: String,
    pub volume: String,
    pub time: String,
    pub side: String, // Manually deserialize into OrderSide::{BUY/SELL}
    pub orderType: String,
    pub misc: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum KrakenPublicData {
    // Ticker(KrakenTickerData),
    OHLC(KrakenOHLCData),
    Trade(KrakenTradeData),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KrakenPublicMessage {
    pub channel_id: i64,
    pub data: KrakenPublicData,
    pub channel_name: String,
    pub pair: String,
}
