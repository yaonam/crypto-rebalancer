use std::collections::HashMap;

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
    Trade(Vec<KrakenTradeData>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KrakenPublicMessage {
    pub channel_id: i64,
    pub data: KrakenPublicData,
    pub channel_name: String,
    pub pair: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KrakenOrderDataDescr {
    pub pair: String,
    // position
    #[serde(rename = "type")]
    pub _type: String,
    pub ordertype: String,
    pub price: String,
    pub price2: String,
    // leverage: String,
    pub order: String,
    // close
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KrakenOrderData {
    // refid: Option<String>,
    // userref: i64,
    pub status: String,
    // opentm: f64,
    // starttm: f64,
    // display_volume: f64,
    // display_volume_remain: f64,
    // expiretm: f64,
    // Ignore contingent for now
    pub descr: Option<KrakenOrderDataDescr>,
    pub vol: Option<String>,
    pub vol_exec: Option<String>,
    pub cost: Option<String>,
    pub fee: Option<String>,
    pub avg_price: Option<String>,
    // ...
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KrakenSequence {
    sequence: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KrakenOpenOrders {
    pub orders: Vec<HashMap<String, KrakenOrderData>>,
    pub channel_name: String,
    pub sequence: KrakenSequence,
}
