use crate::schema::LimitOrder;
use crate::schema::MarketData;
use crate::schema::OrderBookData;
use crate::strategy::Strategy;
use async_trait::async_trait;
use futures_util::stream::{SelectAll, SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[async_trait]
pub trait Broker {
    async fn connect(
        &mut self,
        symbols: Vec<String>,
    ) -> (
        SplitSink<Socket, Message>,
        SplitSink<Socket, Message>,
        String,
    );

    async fn start(&mut self);
}

#[async_trait]
pub trait BrokerStatic: Sync + Send {
    /// Returns the order book for the given symbol.
    async fn get_order_book(client: reqwest::Client, symbol: String) -> OrderBookData
    where
        Self: Sized;

    async fn get_assets() -> f64
    where
        Self: Sized;

    async fn place_order(
        priv_sink: &mut SplitSink<Socket, Message>,
        order: LimitOrder,
        token: String,
    ) where
        Self: Sized;

    async fn cancel_order(order: LimitOrder)
    where
        Self: Sized;
}
