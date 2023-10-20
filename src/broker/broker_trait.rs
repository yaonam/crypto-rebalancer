use crate::schema::LimitOrder;


use crate::strategy::Strategy;
use async_trait::async_trait;
use futures::stream::SplitSink;

use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[async_trait]
pub trait Broker<T: Strategy> {
    async fn connect(
        &mut self,
        symbols: Vec<&str>,
    ) -> (
        SplitSink<Socket, Message>,
        SplitSink<Socket, Message>,
        String,
    );

    fn set_strat(&mut self, strat: T);

    /// Start listening to and forwarding wss messages.
    async fn start(&mut self);
}

#[async_trait]
pub trait BrokerStatic: Sync + Send {
    /// Returns the order book for the given symbol.
    async fn get_mid_price(client: reqwest::Client, symbol: String) -> f64
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
