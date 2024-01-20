use futures_util::stream::SplitSink;
use futures_util::{stream::SelectAll, stream::SplitStream, SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::product::Market;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn connect_public() -> Result<(SplitSink<Socket, Message>, SplitStream<Socket>), Error> {
    connect("wss://ws.kraken.com").await
}

pub async fn connect_private() -> Result<(SplitSink<Socket, Message>, SplitStream<Socket>), Error> {
    connect("wss://ws-auth.kraken.com").await
}

async fn connect(url: &str) -> Result<(SplitSink<Socket, Message>, SplitStream<Socket>), Error> {
    match connect_async(url).await {
        Ok((socket, _)) => {
            println!("Connected to Kraken");
            let (sink, stream) = socket.split();
            Ok((sink, stream))
        }
        Err(e) => {
            println!("Failed to connect to Kraken: {}", e);
            Err(e)
        }
    }
}

pub async fn send(sink: &mut SplitSink<Socket, Message>, message: &str) -> Result<(), Error> {
    // Send the subscription message
    sink.send(Message::Text(message.to_string())).await
}

/// Listens to messages from both public and private streams and calls market.on_message.
pub async fn listener(
    reader1: SplitStream<Socket>,
    reader2: SplitStream<Socket>,
    market: Arc<Mutex<Market>>,
) {
    let mut streams = SelectAll::new();
    streams.push(reader1);
    streams.push(reader2);

    let read_future = streams.for_each(|message| async {
        match message {
            Err(e) => {
                println!("Error reading from stream: {}", e);
                // ignore
            }
            Ok(message) => {
                market.lock().await.on_message(message.to_string()).await;
            }
        }
    });

    read_future.await;
}
