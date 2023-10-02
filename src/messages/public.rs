use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct TickerData {
    a: Vec<serde_json::Value>,
    b: Vec<serde_json::Value>,
    c: Vec<serde_json::Value>,
    v: Vec<serde_json::Value>,
    p: Vec<serde_json::Value>,
    t: Vec<serde_json::Value>,
    l: Vec<serde_json::Value>,
    h: Vec<serde_json::Value>,
    o: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum PublicData {
    Ticker(TickerData),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicMessage {
    channel_id: i64,
    data: PublicData,
    channel_name: String,
    pair: String,
}
