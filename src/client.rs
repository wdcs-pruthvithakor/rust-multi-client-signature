use crate::utils;
use ed25519_dalek::{Signature, Signer, SigningKey};
use futures::StreamExt;
use tokio::time::{timeout, Duration, Instant};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream};

/// Connect to WebSocket server.
async fn connect_to_websocket(
) -> Result<tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>>
{
    let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";
    let (ws_stream, _) = connect_async(url).await?;
    Ok(ws_stream)
}

/// Client process: Fetch prices, calculate average, sign, and send to aggregator.
pub async fn client_process(
    id: usize,
    tx: mpsc::Sender<(usize, f64, Signature)>,
    keypair: SigningKey,
    duration: u64,
) {
    let mut ws_stream = match connect_to_websocket().await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Client {id}: Failed to connect to WebSocket: {e}");
            return;
        }
    };

    println!("Client {id}: Connected to WebSocket.");
    let mut prices: Vec<f64> = Vec::new();
    let start_time = Instant::now();

    while start_time.elapsed().as_secs() < duration {
        let remaining_time = duration.saturating_sub(start_time.elapsed().as_secs());

        // Set a timeout for receiving a message
        let result = timeout(Duration::from_secs(remaining_time), ws_stream.next()).await;

        match result {
            Ok(Some(Ok(Message::Text(text)))) => {
                if let Ok(price) = utils::process_message(&text) {
                    prices.push(price);
                }
            }
            Ok(Some(Err(e))) => {
                eprintln!("Client {id}: WebSocket error: {e}");
                break;
            }
            Ok(None) => {
                eprintln!("Client {id}: WebSocket stream closed.");
                break;
            }
            Err(_) => {
                eprintln!("Client {id}: Timeout reached while waiting for WebSocket message.");
                break;
            }
            _ => {
                break;
            }
        }
    }

    if let Some(avg) = utils::calculate_average(&prices) {
        println!("Client {id}: Average BTC price: {:.4}", avg);

        let message = format!("{}:{}", id, avg);
        let signature = keypair.sign(message.as_bytes());

        let _ = tx.send((id, avg, signature)).await;
        utils::save_client_data(id, &prices, avg)
            .unwrap_or_else(|e| eprintln!("Client {id}: Failed to save data: {e}"));
    } else {
        eprintln!("Client {id}: No data points collected.");
    }
}
