use futures::StreamExt;
use serde_json::Value;
use tokio::{net::TcpStream, sync::mpsc, task};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream};
use std::time::Instant;
use std::fs::File;
use std::io::{self, BufRead, Write, BufReader};
use clap::{Command, Arg};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;

/// Client process: Fetch prices, calculate average, sign, and send to aggregator.
async fn client_process(
    id: usize,
    tx: mpsc::Sender<(usize, f64, Signature)>,
    keypair: SigningKey,
    duration: u64
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
        if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
            if let Ok(price) = process_message(&text) {
                prices.push(price);
            }
        } else {
            eprintln!("Client {id}: Failed to receive message.");
            break;
        }
    }

    if let Some(avg) = calculate_average(&prices) {
        println!("Client {id}: Average BTC price: {:.4}", avg);

        let message = format!("{}:{}", id, avg);
        let signature = keypair.sign(message.as_bytes());

        let _ = tx.send((id, avg, signature)).await;
        save_client_data(id, &prices, avg).unwrap_or_else(|e| eprintln!("Client {id}: Failed to save data: {e}"));
    } else {
        eprintln!("Client {id}: No data points collected.");
    }
}

/// Aggregator process: Compute global average from signed client messages.
async fn aggregator_process(
    mut rx: mpsc::Receiver<(usize, f64, Signature)>,
    num_clients: usize,
    public_keys: Vec<VerifyingKey>
) {
    let mut averages = Vec::new();

    for _ in 0..num_clients {
        if let Some((id, avg, signature)) = rx.recv().await {
            let message = format!("{}:{}", id, avg);
            // To test signature failure case we need to alter the message
            // let mut message = format!("{}:{}", id, avg);
            // if id == 3 {
            //     message = format!("a");
            // }
            if !(1..=num_clients).contains(&id) {eprintln!("Aggregator: Failed to verify signature from client {id}"); continue; }
            if public_keys[id - 1].verify(message.as_bytes(), &signature).is_ok() {
                println!("Aggregator: Verified average from client {id}: {avg:.4}");
                averages.push(avg);
            } else {
                let _ = save_client_error_data(id, format!("Aggregator: Failed to verify signature from client {id}"));
                eprintln!("Aggregator: Failed to verify signature from client {id}");
            }
        }
    }

    if let Some(global_avg) = calculate_average(&averages) {
        println!("Aggregator: Global average BTC price: {:.4}", global_avg);
        save_global_data(&averages, global_avg).unwrap_or_else(|e| eprintln!("Aggregator: Failed to save global data: {e}"));
    } else {
        eprintln!("Aggregator: No valid averages received.");
    }
}

/// Generate a keypair for each client.
fn generate_keypairs(num_clients: usize) -> Vec<SigningKey> {
    (0..num_clients).map(|_| SigningKey::generate(&mut OsRng)).collect()
}

/// Connect to WebSocket server.
async fn connect_to_websocket() -> Result<tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
    let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";
    let (ws_stream, _) = connect_async(url).await?;
    Ok(ws_stream)
}

/// Process WebSocket message to extract price.
fn process_message(text: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let json: Value = serde_json::from_str(text)?;
    if let Some(price) = json.get("p") {
        price.as_str().unwrap().parse::<f64>().map_err(|e| e.into())
    } else {
        Err("No price field found".into())
    }
}

/// Calculate the average of a vector of numbers.
fn calculate_average(prices: &Vec<f64>) -> Option<f64> {
    if prices.is_empty() {
        None
    } else {
        Some(prices.iter().sum::<f64>() / prices.len() as f64)
    }
}

/// Save individual client data to file.
fn save_client_data(id: usize, prices: &Vec<f64>, average: f64) -> std::io::Result<()> {
    let mut file = File::create(format!("client_{id}_data.txt"))?;
    writeln!(file, "Prices: {:?}\nAverage: {:.4}", prices, average)?;
    Ok(())
}

fn save_client_error_data(id: usize, message: String) -> std::io::Result<()> {
    let mut file = File::create(format!("client_{id}_data.txt"))?;
    writeln!(file, "Error: {}", message)?;
    Ok(())
}

/// Save global aggregator data to file.
fn save_global_data(averages: &Vec<f64>, global_average: f64) -> std::io::Result<()> {
    let mut file = File::create("global_data.txt")?;
    writeln!(file, "Client Averages: {:?}\nGlobal Average: {:.4}", averages, global_average)?;
    Ok(())
}

/// Parse the command-line arguments
fn parse_arguments() -> clap::ArgMatches {
    Command::new("WebSocket Listener")
        .version("1.0")
        .author("Pruthvi Thakor")
        .about("Listens to the WebSocket for BTC/USDT prices")
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("Specifies the mode of operation")
                .default_value("read"),
        )
        .arg(
            Arg::new("times")
                .short('t')
                .long("times")
                .value_name("NUMBER")
                .help("The number of seconds to listen")
                .default_value("1"),
            )
            .get_matches()
        }
        
/// Prints the data after reading it from file
fn read_mode(num_clients: usize) -> io::Result<()> {
    println!("Reading prices data ...\n");
    let mut files: Vec<String> = Vec::with_capacity(num_clients+1);
    for i in 1..=num_clients {
        files.push(format!("client_{}_data.txt", i));
    }
    files.push(String::from("global_data.txt"));
    'file_loop: for file_path in files.iter() {
        // Attempt to open the file
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to open {}: {}", file_path, err);
                break 'file_loop; // Exit the loop on error
            }
        };
        println!("\nReading file: {}\n", file_path);
        let reader = BufReader::new(file);

        // Read the file line by line
        for line in reader.lines() {
            match line {
                Ok(content) => println!("{}", content),
                Err(err) => {
                    eprintln!("Error reading a line in {}: {}", file_path, err);
                    break 'file_loop; // Exit the loop on error
                }
            }
        }
    }

    Ok(())
}


#[tokio::main]
async fn main() {
    let num_clients: usize = 5;
    let matches = parse_arguments();

    // Extract the mode and times arguments
    let mode = matches.get_one::<String>("mode").unwrap();
    let times: u64 = matches
        .get_one::<String>("times")
        .unwrap()
        .parse()
        .unwrap_or(1);

    // Print the parsed arguments
    println!("Mode: {}", mode);


    // Start the WebSocket listener in the "cache" mode
    match mode.as_str() {
        "cache" => {
            let num_clients: usize = 5;


            let keypairs = generate_keypairs(num_clients);
            let public_keys: Vec<VerifyingKey> = keypairs.iter().map(|kp| kp.verifying_key()).collect();

            let (tx, rx) = mpsc::channel(num_clients);
            let aggregator = task::spawn(aggregator_process(rx, num_clients, public_keys));

            let mut clients = Vec::new();
            for (id, keypair) in keypairs.into_iter().enumerate() {
                let tx_clone = tx.clone();
                clients.push(task::spawn(client_process(id + 1, tx_clone, keypair, times)));
            }
            println!("Will listen for {} seconds.", times);
            for client in clients {
                let _ = client.await;
            }

            let _ = aggregator.await;
        },
        "read" => read_mode(num_clients).expect("Failed to read price data"),
        _ => eprintln!("Invalid mode: {mode}. Use --mode=cache or --mode=read.")
    }
    
}
