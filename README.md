# WebSocket Listener for BTC/USDT Prices with Signing and Aggregation

This Rust project connects to Binance's WebSocket API to listen for real-time BTC/USDT trade prices. It supports two operational modes: **cache mode** and **read mode**. In **cache mode**, multiple simulated clients fetch and process price data concurrently, compute averages, sign their results, and send them to an aggregator. The aggregator verifies the signed averages and computes a global average. In **read mode**, the program displays previously saved data.

---

## Features

- **Cache Mode**: 
  - Simulates multiple clients that fetch price updates from Binance's WebSocket.
  - Each client computes the average BTC price from received messages.
  - Signs the computed averages using Ed25519 keys and sends them to an aggregator.
  - The aggregator verifies the signatures and computes a global average.
  - Saves client data (prices and averages) and global data to files.
  
- **Read Mode**:
  - Reads and displays the saved data from the files generated in **cache mode**.
  
- **Digital Signatures**:
  - Each client signs its computed average price to ensure authenticity.
  - The aggregator verifies the signatures before accepting the data.

---

## Requirements

### Prerequisites

1. **Rust**: Install Rust from [here](https://www.rust-lang.org/tools/install).
2. **Dependencies**:
   - `tokio`: Asynchronous runtime.
   - `tokio-tungstenite`: WebSocket client library.
   - `serde`, `serde_json`: JSON processing.
   - `clap`: Command-line argument parsing.
   - `ed25519-dalek`: Digital signature implementation.
   - `rand`: Random number generation.

Dependencies are specified in the `Cargo.toml` file.

---

## Installation

1. Clone this repository:

   ```bash
   git clone https://github.com/wdcs-pruthvithakor/rust-multi-client-signature.git
   ```

2. Navigate into the project directory:

   ```bash
   cd rust-multi-client-signature
   ```

3. Build the project:

   ```bash
   cargo build --release
   ```

---

## Usage

### **1. Cache Mode**

In **cache mode**, the program runs multiple clients, collects real-time BTC/USDT prices for a specified duration, computes averages, and sends the signed data to an aggregator. The results are saved to files.

Run the program in **cache mode**:

```bash
cargo run -- --mode cache --times <seconds>
```

- `--mode cache`: Specifies the program should run in cache mode.
- `--times <seconds>`: Sets the duration (in seconds) for listening to WebSocket messages (default is 1 second).

#### Example:
```bash
cargo run -- --mode cache --times 10
```

This starts 5 clients listening for 10 seconds, calculates averages, and saves the results.

---

### **2. Read Mode**

In **read mode**, the program reads and displays previously saved data from files.

Run the program in **read mode**:

```bash
cargo run -- --mode read
```

#### Files Read:
- `client_1_data.txt`, `client_2_data.txt`, ..., `client_5_data.txt`: Contains individual client data.
- `global_data.txt`: Contains global averages computed by the aggregator.

---

## File Outputs

- **client_X_data.txt** (e.g., `client_1_data.txt`):
  Contains the list of prices received and the computed average.

  Example:
  ```
  Prices: [34912.45, 34914.32, 34910.12]
  Average: 34912.30
  ```

- **global_data.txt**:
  Contains the averages from all clients and the global average.

  Example:
  ```
  Client Averages: [34912.30, 34915.12, 34911.45]
  Global Average: 34912.29
  ```

---

## Code Overview

### **Main Components**
- **Client Process**:
  - Listens to the Binance WebSocket for BTC/USDT prices.
  - Calculates the average price from received data.
  - Signs the result with Ed25519 keys and sends it to the aggregator.

- **Aggregator Process**:
  - Verifies the signatures from all clients.
  - Computes the global average from verified client averages.
  - Saves global results to a file.

### **Key Functions**:
- `connect_to_websocket`: Establishes a WebSocket connection.
- `process_message`: Parses WebSocket messages to extract BTC prices.
- `calculate_average`: Computes the average of a list of numbers.
- `save_client_data`: Saves individual client results to a file.
- `save_global_data`: Saves global results (client averages and global average) to a file.
- `parse_arguments`: Parses and validates command-line arguments.
- `read_mode`: Reads and displays previously saved data.

---

## Error Handling

- **WebSocket Connection Errors**: Logs errors if clients fail to connect or receive messages.
- **Signature Verification Errors**: The aggregator logs any unverified signatures and discards invalid data.
- **File Handling Errors**: Displays error messages if reading or writing files fails.

---

## Example Workflow

1. **Run Cache Mode**:
   ```bash
   cargo run -- --mode cache --times 10
   ```
   - Listens for price updates for 10 seconds.
   - Saves individual and global data to files.

2. **Read Saved Data**:
   ```bash
   cargo run -- --mode read
   ```
   - Reads and displays the saved results from the files.

---

## Contribution

Contributions are welcome! Feel free to fork the repository, open issues, or submit pull requests.

---

## License

This project is licensed under the [MIT License](LICENSE).

---

## Author

**Pruthvi Thakor**  
Contact: https://github.com/wdcs-pruthvithakor