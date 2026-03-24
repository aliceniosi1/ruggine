use common::Message;
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc::{unbounded_channel, UnboundedReceiver}, Mutex};
use serde_json;
use std::sync::Arc;
use once_cell::sync::OnceCell;

static WRITER: OnceCell<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>> = OnceCell::new();

pub async fn connect_and_listen(addr: &str) -> Result<UnboundedReceiver<Message>, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(addr).await?;
    let (reader, writer) = stream.into_split();

    let writer_arc = Arc::new(Mutex::new(writer));
    WRITER.set(writer_arc.clone()).map_err(|_| "WRITER già inizializzato")?;

    let (tx, rx) = unbounded_channel();

    tokio::spawn(async move {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        while buf_reader.read_line(&mut line).await.unwrap_or(0) > 0 {
            let trimmed = line.trim_end();
            if let Ok(msg) = serde_json::from_str::<Message>(trimmed) {
                let _ = tx.send(msg);
            }
            line.clear();
        }
    });

    Ok(rx)
}

pub async fn send_message(msg: &Message) {
    let json = serde_json::to_string(msg).unwrap() + "\n";
    if let Some(writer) = WRITER.get() {
        let mut w = writer.lock().await;
        let _ = w.write_all(json.as_bytes()).await;
        println!("🚀 Inviato linea JSON: {}", json.trim());
    } else {
        eprintln!("❌ WRITER non inizializzato!");
    }
}
