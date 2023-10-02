use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Trade {
    order_tx_id: String,
    pos_tx_id: String,
    pair: String,
    time: f64,
    type_: String,
    ordertype: String,
    price: String,
    cost: String,
    fee: String,
    vol: String,
    margin: String,
    user_ref: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TradeSet(HashMap<String, Trade>);

#[derive(Serialize, Deserialize, Debug)]
struct Sequence {
    sequence: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct OwnTradesData {
    trade: TradeSet,
    channel_name: String,
    sequence: Sequence,
}
