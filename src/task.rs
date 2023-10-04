use crate::account::{Portfolio, Signer};
use crate::product::Market;
use crate::websocket::{connect_private, connect_public, listener, send};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Helps spawn task by fetching ws token. Returns a JoinHandle.
pub async fn spawn(
    pair: String,
    portfolio: Arc<Mutex<Portfolio>>,
    signer: Arc<Mutex<Signer>>,
) -> JoinHandle<()> {
    let token = { signer.lock().await.get_ws_token().await };
    tokio::spawn(start(pair, portfolio, token))
}

pub async fn start(pair: String, portfolio: Arc<Mutex<Portfolio>>, token: String) {
    let (mut pub_sink, pub_reader) = connect_public().await.unwrap();
    let (mut priv_sink, priv_reader) = connect_private().await.unwrap();

    // Sub to ticker
    let message = json!(
    {
        "event": "subscribe",
        "pair": [pair],
        "subscription": {
            "name": "ticker"
        }
    })
    .to_string();
    send(&mut pub_sink, &message).await.unwrap();

    // Sub to open orders
    let message = json!(
    {
        "event": "subscribe",
        "subscription": {
            "name": "openOrders",
            "token": token,
        }
    })
    .to_string();
    send(&mut priv_sink, &message).await.unwrap();

    // let message = json!(
    //     {
    //         "event": "addOrder",
    //         "ordertype": "limit",
    //         "pair": "STORJ/USD",
    //         "price": "1638.84",
    //         "token": token,
    //         "type": "buy",
    //         "volume": "0.012074779107009711"
    //     }
    // )
    // .to_string();
    // println!("Sending: {}", message);
    // send(&mut priv_sink, &message).await.unwrap();
    // let message = json!(
    //     {
    //         "event": "addOrder",
    //         "ordertype": "limit",
    //         "pair": "ETH/USD",
    //         "price": "1638.84",
    //         "token": token,
    //         "type": "buy",
    //         "volume": "0.012074779107009711"
    //     }
    // )
    // .to_string();
    // println!("Sending: {}", message);
    // send(&mut priv_sink, &message).await.unwrap();

    let market = Arc::new(Mutex::new(Market::new(pair, portfolio, priv_sink, token)));
    listener(pub_reader, priv_reader, market.clone()).await;
}
