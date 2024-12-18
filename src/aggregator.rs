use crate::utils;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration, Instant};

/// Aggregator process: Compute global average from signed client messages.
pub async fn aggregator_process(
    mut rx: mpsc::Receiver<(usize, f64, Signature)>,
    num_clients: usize,
    public_keys: Vec<VerifyingKey>,
    duration: u64,
) {
    let mut averages = Vec::new();
    let start_time = Instant::now();
    let mut clients_verified = 0;

    while start_time.elapsed().as_secs() < (duration + 5) {
        // Calculate the remaining time to adjust the timeout dynamically
        let remaining_time = (duration + 5).saturating_sub(start_time.elapsed().as_secs());

        // Set a timeout for receiving a message
        let result = timeout(Duration::from_secs(remaining_time), rx.recv()).await;

        match result {
            Ok(Some((id, avg, signature))) => {
                let message = format!("{}:{}", id, avg);
                // To test signature failure case we need to alter the message
                // let mut message = format!("{}:{}", id, avg);
                // if id == 3 {
                //     message = format!("a");
                // }
                if !(1..=num_clients).contains(&id) {
                    eprintln!("Aggregator: Failed to verify signature from client {id}");
                    continue;
                }

                if public_keys[id - 1]
                    .verify(message.as_bytes(), &signature)
                    .is_ok()
                {
                    println!("Aggregator: Verified average from client {id}: {avg:.4}");
                    clients_verified += 1;
                    averages.push(avg);

                    if clients_verified >= num_clients {
                        break;
                    }
                } else {
                    let _ = utils::save_client_error_data(
                        id,
                        format!("Aggregator: Failed to verify signature from client {id}"),
                    );
                    eprintln!("Aggregator: Failed to verify signature from client {id}");
                }
            }
            Ok(None) => {
                eprintln!("Aggregator: No more messages from clients.");
                break;
            }
            Err(_) => {
                eprintln!("Aggregator: Timeout while waiting for client messages.");
                break;
            }
        }
    }

    if let Some(global_avg) = utils::calculate_average(&averages) {
        println!("Aggregator: Global average BTC price: {:.4}", global_avg);
        utils::save_global_data(&averages, global_avg)
            .unwrap_or_else(|e| eprintln!("Aggregator: Failed to save global data: {e}"));
    } else {
        eprintln!("Aggregator: No valid averages received.");
    }
}
