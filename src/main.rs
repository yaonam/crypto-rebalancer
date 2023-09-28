use std::process::exit;

use futures_util::stream::SplitSink;
use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[tokio::main]
async fn main() {
    let (mut sink, reader) = connect().await.unwrap();
    subscribe(&mut sink).await.unwrap();
    listener(reader).await;
}

async fn connect() -> Result<(SplitSink<Socket, Message>, SplitStream<Socket>), Error> {
    match connect_async("wss://ws.kraken.com").await {
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

async fn subscribe(sink: &mut SplitSink<Socket, Message>) -> Result<(), Error> {
    let subscription_message = r#"
    {
        "event": "subscribe",
        "pair": ["XBT/USD", "XBT/EUR"],
        "subscription": {
            "name": "ticker"
        }
    }
    "#;

    // Send the subscription message
    sink.send(Message::Text(subscription_message.to_string()))
        .await
}

async fn listener(reader: SplitStream<Socket>) {
    let read_future = reader.for_each(|message| async {
        let data = message.unwrap().to_string();
        println!("{}", data);
    });

    read_future.await;
}

fn on_data() {}

fn set_allocation() {}
