use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct OrderDataDescr {
    pair: String,
    // position
    #[serde(rename = "type")]
    _type: String,
    ordertype: String,
    price: String,
    price2: String,
    // leverage: String,
    order: String,
    // close
}

#[derive(Serialize, Deserialize, Debug)]
struct OrderData {
    // refid: Option<String>,
    // userref: i64,
    status: String,
    // opentm: f64,
    // starttm: f64,
    // display_volume: f64,
    // display_volume_remain: f64,
    // expiretm: f64,
    // Ignore contingent for now
    descr: OrderDataDescr,
    vol: String,
    vol_exec: String,
    cost: String,
    fee: String,
    avg_price: String,
    // ...
}

#[derive(Serialize, Deserialize, Debug)]
struct Sequence {
    sequence: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenOrders {
    orders: Vec<HashMap<String, OrderData>>,
    channel_name: String,
    sequence: Sequence,
}
