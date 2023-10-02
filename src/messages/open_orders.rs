use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderDataDescr {
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
pub struct OrderData {
    // refid: Option<String>,
    // userref: i64,
    pub status: String,
    // opentm: f64,
    // starttm: f64,
    // display_volume: f64,
    // display_volume_remain: f64,
    // expiretm: f64,
    // Ignore contingent for now
    pub descr: OrderDataDescr,
    pub vol: String,
    pub vol_exec: String,
    pub cost: String,
    pub fee: String,
    pub avg_price: String,
    // ...
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sequence {
    sequence: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenOrders {
    pub orders: Vec<HashMap<String, OrderData>>,
    pub channel_name: String,
    pub sequence: Sequence,
}
