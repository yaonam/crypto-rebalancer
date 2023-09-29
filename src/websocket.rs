use futures_util::stream::SplitSink;
use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type Callback = fn(String);

pub async fn connect() -> Result<(SplitSink<Socket, Message>, SplitStream<Socket>), Error> {
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

pub async fn subscribe(sink: &mut SplitSink<Socket, Message>, message: &str) -> Result<(), Error> {
    // Send the subscription message
    sink.send(Message::Text(message.to_string())).await
}

pub async fn listener(reader: SplitStream<Socket>, callback: Callback) {
    let read_future = reader.for_each(|message| async {
        let data = message.unwrap().to_string();
        callback(data);
    });

    read_future.await;
}
