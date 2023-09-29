use crate::product::Product;
use crate::websocket::{connect, listener, subscribe};
use serde_json::json;

pub async fn start(pair: String) {
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
    subscribe(&mut sink, &message).await.unwrap();
    let prod = Product::new(pair);
    listener(reader, Box::new(move |msg| prod.on_message(msg))).await;
}
