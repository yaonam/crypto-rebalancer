use super::broker_trait::{Broker, BrokerStatic};
use crate::strategy::Strategy;
use crate::websocket::connect;
use async_trait::async_trait;
use futures_util::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const WS_URL: &str = "wss://ws.kraken.com";

pub struct Backtest {}

#[async_trait]
impl<T: Strategy> Broker<T> for Backtest {
    async fn connect(
        &mut self,
        symbols: Vec<&str>,
    ) -> (
        SplitSink<Socket, Message>,
        SplitSink<Socket, Message>,
        String,
    ) {
        let (mut pub_sink1, _) = connect(WS_URL).await.unwrap();
        let (mut pub_sink2, _) = connect(WS_URL).await.unwrap();
        (pub_sink1, pub_sink2, "".to_string())
    }

    fn set_strat(&mut self, strat: T) {}

    async fn start(&mut self) {}
}
