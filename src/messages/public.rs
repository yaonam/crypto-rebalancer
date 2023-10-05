use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct TickerData {
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
#[serde(untagged)]
pub enum PublicData {
    Ticker(TickerData),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicMessage {
    pub channel_id: i64,
    pub data: PublicData,
    pub channel_name: String,
    pub pair: String,
}
