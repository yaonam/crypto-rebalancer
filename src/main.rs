mod websocket;

use crate::websocket::{connect, listener, subscribe};

#[tokio::main]
async fn main() {
    let (mut sink, reader) = connect().await.unwrap();
    let message = r#"
    {
        "event": "subscribe",
        "pair": ["ETH/USD"],
        "subscription": {
            "name": "book"
        }
    }
    "#;
    subscribe(&mut sink, message).await.unwrap();
    listener(reader, on_message).await;
}

fn on_message(message: String) {
    println!("{}", message);
}

fn on_data() {}

fn on_order_filled() {}

fn record_price() {}

fn get_reserve_price() {}

fn get_optimal_spread() {}

fn get_last_price() {}

fn set_last_price() {}

fn get_allocation() {}

fn set_allocation() {}

fn get_bid_size() {}

fn get_ask_size() {}

fn get_dist_to_target() {}

fn get_volatility() {}

fn get_order_depth() {}
