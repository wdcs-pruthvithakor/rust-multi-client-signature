use ed25519_dalek::VerifyingKey;
use multi_client_signature::{aggregator, client, utils};
use tokio::{sync::mpsc, task};

#[tokio::main]
async fn main() {
    let num_clients: usize = 5;
    let matches = utils::parse_arguments();

    // Extract the mode and times arguments
    let mode = if let Some(mode) = matches.get_one::<String>("mode") {
        mode.as_str()
    } else {
        ""
    };

    // Print the parsed arguments
    println!("Mode: {}", mode);

    // Start the WebSocket listener in the "cache" mode
    match mode {
        "cache" => {
            let num_clients: usize = 5;
            let default_mode = String::default();
            let times: u64 = matches
            .get_one::<String>("times")
            .unwrap_or(&default_mode)
            .parse()
            .unwrap_or_else(|_|{ eprintln!("Failed to parse input time value, please enter valid seconds, taking default 1 to calculate.."); 1});

            let keypairs = utils::generate_keypairs(num_clients);
            let public_keys: Vec<VerifyingKey> =
                keypairs.iter().map(|kp| kp.verifying_key()).collect();

            let (tx, rx) = mpsc::channel(num_clients);
            let aggregator = task::spawn(aggregator::aggregator_process(
                rx,
                num_clients,
                public_keys,
                times,
            ));

            let mut clients = Vec::new();
            for (id, keypair) in keypairs.into_iter().enumerate() {
                let tx_clone = tx.clone();
                clients.push(task::spawn(client::client_process(
                    id + 1,
                    tx_clone,
                    keypair,
                    times,
                )));
            }
            println!("Will listen for {} seconds.", times);
            for client in clients {
                let _ = client.await;
            }

            let _ = aggregator.await;
        }
        "read" => utils::read_mode(num_clients).expect("Failed to read price data"),
        _ => eprintln!("Invalid mode: {mode}. Use --mode=cache or --mode=read."),
    }
}
