use std::sync::Arc;

use anyhow::{Context, anyhow};
use futures::{SinkExt, StreamExt};

use rustls::ClientConfig;

use serde_json::Value;

use tokio::{
    net::TcpStream,
    sync::{Mutex, mpsc},
};

use tokio_tungstenite::{
    Connector, MaybeTlsStream, WebSocketStream, connect_async_tls_with_config, tungstenite::Message,
};

use crate::websocket::tls::NoCertVerify;

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct WebSocketClient {
    tx: mpsc::UnboundedSender<Message>,
    rx: Arc<Mutex<mpsc::UnboundedReceiver<Value>>>,
}

impl Clone for WebSocketClient {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: self.rx.clone(),
        }
    }
}

impl WebSocketClient {
    pub async fn connect(addr: &str, api_key: &str) -> anyhow::Result<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default();

        let tls = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertVerify))
            .with_no_client_auth();

        let url = format!("wss://{}/ws?key={}", addr, api_key);

        let (ws, _) = connect_async_tls_with_config(
            &url,
            None,
            false,
            Some(Connector::Rustls(Arc::new(tls))),
        )
        .await
        .with_context(|| format!("failed connecting to {}", url))?;

        let (tx, outgoing_rx) = mpsc::unbounded_channel::<Message>();

        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel::<Value>();

        tokio::spawn(io_loop(ws, outgoing_rx, incoming_tx));

        Ok(Self {
            tx,
            rx: Arc::new(Mutex::new(incoming_rx)),
        })
    }

    pub fn send(&self, value: Value) -> anyhow::Result<()> {
        self.tx
            .send(Message::Text(value.to_string().into()))
            .map_err(|_| anyhow!("socket closed"))
    }

    /// Wait until a message arrives.
    pub async fn read(&self) -> Option<Value> {
        let mut rx = self.rx.lock().await;

        rx.recv().await
    }

    /// Return immediately if no message exists.
    #[allow(dead_code)]
    pub async fn try_read(&self) -> Option<Value> {
        let mut rx = self.rx.lock().await;

        rx.try_recv().ok()
    }

    /// Drain all queued messages without blocking.
    #[allow(dead_code)]
    pub async fn drain(&self) -> Vec<Value> {
        let mut messages = Vec::new();

        let mut rx = self.rx.lock().await;

        while let Ok(msg) = rx.try_recv() {
            messages.push(msg);
        }

        messages
    }
}

async fn io_loop(
    ws: Ws,
    mut outgoing_rx: mpsc::UnboundedReceiver<Message>,
    incoming_tx: mpsc::UnboundedSender<Value>,
) {
    let (mut write, mut read) = ws.split();

    loop {
        tokio::select! {
            Some(msg) = outgoing_rx.recv() => {
                if write.send(msg).await.is_err() {
                    break;
                }
            }

            incoming = read.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(value) =
                            serde_json::from_str::<Value>(&text)
                        {
                            let _ =
                                incoming_tx.send(value);
                        }
                    }

                    Some(Ok(Message::Close(_))) => {
                        break;
                    }

                    Some(Err(err)) => {
                        println!(
                            "WebSocket error: {}",
                            err
                        );
                        break;
                    }

                    None => {
                        break;
                    }

                    _ => {}
                }
            }
        }
    }
}
