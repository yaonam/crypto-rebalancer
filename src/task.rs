use crate::account::Portfolio;
use crate::product::Market;
use crate::websocket::{connect, listener, send};
use serde_json::json;
use std::sync::{Arc, Mutex};

pub async fn start(pair: String, portfolio: Arc<Mutex<Portfolio>>) {
    let (mut sink, reader) = connect().await.unwrap();
    let message = json!(
    {
        "event": "subscribe",
        "pair": [pair],
        "subscription": {
            "name": "ticker"
        }
    })
    .to_string();
    send(&mut sink, &message).await.unwrap();
    let prod = Market::new(pair, portfolio);
    listener(reader, Box::new(move |msg| prod.on_message(msg))).await;
}
