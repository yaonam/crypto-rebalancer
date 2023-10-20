use std::collections::HashMap;

use super::broker_trait::{Broker, BrokerStatic};
use crate::schema::LimitOrder;
use crate::strategy::Strategy;
use crate::websocket::connect;
use async_trait::async_trait;
use csv::Reader;
use futures_util::stream::SplitSink;
use futures_util::StreamExt;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::TlsConnector;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{accept_async, MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const URL: &str = "127.0.0.1:8080";
const WS_URL: &str = "wss://127.0.0.1:8080";

#[derive(Debug, Deserialize)]
struct FileOHCLData {
    timestamp: u16,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    trades: u16,
}

pub struct Backtest<T: Strategy> {
    listener: TcpListener,
    balances: HashMap<String, f64>,
    strat: Option<Arc<T>>,
}

impl<T: Strategy> Backtest<T> {
    pub async fn new() -> Self {
        Backtest {
            listener: TcpListener::bind(URL).await.unwrap(),
            balances: HashMap::new(),
            strat: Option::None,
        }
    }
}

#[async_trait]
impl<T: Strategy> Broker<T> for Backtest<T> {
    async fn connect(
        &mut self,
        symbols: Vec<&str>,
    ) -> (
        SplitSink<Socket, Message>,
        SplitSink<Socket, Message>,
        String,
    ) {
        println!("Got here");
        let (pub_sink1, _) = connect(WS_URL).await.unwrap();
        println!("Got here");
        let (pub_sink2, _) = connect(WS_URL).await.unwrap();
        (pub_sink1, pub_sink2, String::new())
    }

    fn set_strat(&mut self, strat: T) {}

    async fn start(&mut self) {
        // Alternate between ohlc and trade data
        // Whichever entry is first (timestamp-wise), call on_data/on_order
        // Keep track of balances and orders
        println!("Backtest starting...");
        let mut rdr = Reader::from_path("backtest/ETHUSD_5.csv").unwrap();
        println!("Reading file...");
        for ohcl in rdr.deserialize() {
            let file_ohcl: FileOHCLData = ohcl.unwrap();
            println!("{:?}", file_ohcl);
        }

        while let Ok((stream, _)) = self.listener.accept().await {
            // let peer = stream
            //     .peer_addr()
            //     .expect("connected streams should have a peer address");

            // tokio::spawn(accept_connection(peer, stream));
            println!("Peer connected!");
        }
    }
}

pub struct BacktestStatic {}

#[async_trait]
impl BrokerStatic for BacktestStatic {
    async fn get_mid_price(client: reqwest::Client, symbol: String) -> f64 {
        0.0
    }

    async fn place_order(
        priv_sink: &mut SplitSink<Socket, Message>,
        order: LimitOrder,
        token: String,
    ) {
    }

    async fn cancel_order(order: LimitOrder) {}
}
