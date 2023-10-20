use super::broker_trait::{Broker, BrokerStatic};
use crate::schema::LimitOrder;
use crate::strategy::Strategy;
use crate::websocket::connect;
use async_trait::async_trait;
use csv::Reader;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{accept_async, MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const URL: &str = "127.0.0.1:8080";
const WS_URL: &str = "wss://127.0.0.1:8080";

const INIT_BAL_USD: f64 = 10000.0;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum BacktestMsg {
    Init(Vec<String>),
}

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

pub struct BacktestPortfolio {
    listener: TcpListener,
    balances: HashMap<String, f64>,
}

impl BacktestPortfolio {
    pub async fn new() -> Self {
        BacktestPortfolio {
            listener: TcpListener::bind(URL).await.unwrap(),
            balances: HashMap::new(),
        }
    }

    pub async fn start(&mut self) {
        // Alternate between ohlc and trade data
        // Whichever entry is first (timestamp-wise), call on_data/on_order
        // Keep track of balances and orders
        println!("Backtest starting...");
        let current_dir = env::current_dir().unwrap();
        println!("Current directory: {:?}", current_dir);
        let path = Path::new("src/broker/backtest/ETHUSD_1440.csv");
        let mut rdr = Reader::from_path(path).unwrap();
        println!("Reading file...");
        for ohcl in rdr.deserialize() {
            let file_ohcl: (u32, f64, f64, f64, f64, f64, u32) = ohcl.unwrap();
            println!("{:?}", file_ohcl);
        }

        // Pub stream
        let (pub_stream, _) = self.listener.accept().await.unwrap();
        let _pub_ws = accept_async(pub_stream).await.unwrap();

        // Priv stream
        let (priv_stream, _) = self.listener.accept().await.unwrap();
        let mut priv_ws = accept_async(priv_stream).await.unwrap();
        while let Some(msg) = priv_ws.next().await {
            let msg = msg.unwrap();
            let s = msg.to_text().unwrap();
            match serde_json::from_str::<BacktestMsg>(s).unwrap() {
                BacktestMsg::Init(_symbols) => {}
            }
        }
    }

    fn init(&mut self, _symbols: Vec<String>) {}
}

pub struct Backtest<T: Strategy> {
    strat: Option<Arc<T>>,
}

impl<T: Strategy> Backtest<T> {
    pub async fn new() -> Self {
        Backtest {
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
        let (pub_sink, _) = connect(WS_URL).await.unwrap();
        let (mut priv_sink, _) = connect(WS_URL).await.unwrap();

        // Init balances
        priv_sink
            .send(Message::Text(format!("{:?}", symbols)))
            .await
            .unwrap();

        (pub_sink, priv_sink, String::new())
    }

    fn set_strat(&mut self, _strat: T) {}

    async fn start(&mut self) {}
}

pub struct BacktestStatic {}

#[async_trait]
impl BrokerStatic for BacktestStatic {
    async fn get_mid_price(_client: reqwest::Client, _symbol: String) -> f64 {
        0.0
    }

    async fn place_order(
        _priv_sink: &mut SplitSink<Socket, Message>,
        _order: LimitOrder,
        _token: String,
    ) {
    }

    async fn cancel_order(_order: LimitOrder) {}
}
