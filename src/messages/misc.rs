use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemStatus {
    event: String,
    status: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscriptionStatus {
    // channel_id: Option<i64>,
    #[serde(rename = "channelName")]
    channel_name: String,
    event: String,
    pair: Option<String>,
    status: String,
    subscription: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Heartbeat {
    event: String,
}
