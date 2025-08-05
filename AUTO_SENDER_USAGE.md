# Automatic UDP Packet Sender

The `auto_sender` binary provides automatic UDP packet transmission with configurable duration and rate, utilizing the existing UDP channel's data integrity features.

## Features

- **Configurable Duration**: Send packets for XXX seconds (configurable via `--duration`)
- **Configurable Rate**: Send YYY packets per second (configurable via `--rate`)
- **Data Integrity**: Utilizes existing UDP channel features including:
  - Acknowledgment (ACK) handling
  - Automatic retransmission on request
  - Flow control with configurable send window
  - Sequence number tracking
- **Statistics**: Real-time and final transmission statistics
- **Flow Control**: Prevents overwhelming the receiver with configurable window size

## Usage

### Basic Usage

Send packets for 10 seconds at 5 packets per second to localhost:8080:

```bash
cargo run --bin auto_sender
```

### Custom Configuration

Send packets for 30 seconds at 20 packets per second with verbose output:

```bash
cargo run --bin auto_sender -- \
    --duration 30 \
    --rate 20 \
    --verbose \
    --target 192.168.1.100:8080 \
    --window-size 50
```

### Command Line Options

```
Options:
  -t, --target <TARGET>            Target address to send packets to (format: IP:PORT) [default: 127.0.0.1:8080]
  -b, --bind <BIND>                Local bind address (format: IP:PORT) [default: 127.0.0.1:0]
  -d, --duration <DURATION>        Duration to send packets in seconds (XXX seconds) [default: 10]
  -r, --rate <RATE>                Packets per second rate (YYY packages per second) [default: 5]
  -m, --message <MESSAGE>          Message content prefix (sequence number will be appended) [default: "Auto message"]
      --window-size <WINDOW_SIZE>  Send window size for flow control [default: 10]
      --ack-timeout <ACK_TIMEOUT>  Acknowledgment timeout in milliseconds [default: 1000]
  -v, --verbose                    Print statistics during sending
  -h, --help                       Print help
  -V, --version                    Print version
```

## Examples

### Example 1: Basic Test Run

```bash
# Terminal 1 - Start receiver
cargo run --bin receiver

# Terminal 2 - Send 20 packets over 10 seconds (2 packets/sec)
cargo run --bin auto_sender -- --duration 10 --rate 2 --verbose
```

**Expected Output:**
```
🚀 UDP Auto Sender Starting...
   Target: 127.0.0.1:8080
   Duration: 10 seconds
   Rate: 2 packets/second
...
📊 Transmission Complete!
   Packets sent: 20
   Actual rate: 2.00 packets/second
   ✅ All messages acknowledged
```

### Example 2: High-Rate Testing

```bash
# Send 1000 packets over 10 seconds (100 packets/sec) with large window
cargo run --bin auto_sender -- \
    --duration 10 \
    --rate 100 \
    --window-size 50 \
    --message "High rate test" \
    --verbose
```

### Example 3: Long Duration Testing

```bash
# Send packets for 5 minutes at 10 packets/sec
cargo run --bin auto_sender -- \
    --duration 300 \
    --rate 10 \
    --target 192.168.1.100:8080
```

## Integration with Existing UDP Channel

The `auto_sender` leverages the existing UDP channel library (`udp_channal_demo`) which provides:

1. **Flow Control**: Prevents buffer overflow with configurable send windows
2. **Acknowledgments**: Each DATA packet receives an ACK from the receiver
3. **Retransmission**: Automatic resending of packets on RQST (retransmission request)
4. **Sequence Tracking**: Each packet has a unique sequence number
5. **Error Handling**: Comprehensive error handling for network issues

## Performance Considerations

- **Rate Limits**: High rates (>100 packets/sec) may require larger window sizes
- **Network Capacity**: Consider network bandwidth when setting high rates
- **Flow Control**: The sender will block if the send window is full (prevents overwhelming receiver)
- **Memory Usage**: Larger windows require more memory for tracking in-flight messages

## Monitoring and Diagnostics

### Verbose Mode Output

With `--verbose` flag, you'll see:

```
📊 Flow Control - In-flight: 5/10, Available window: 5
📤 Sent packet #123 (seq: 123): "Auto message #123"
```

### Final Statistics

After completion:

```
📊 Transmission Complete!
   Duration: 10.00 seconds
   Packets sent: 100
   Packets failed: 0
   Actual rate: 10.00 packets/second
   Target rate: 10 packets/second
   
📊 Final Flow Control Stats:
   In-flight messages: 0/10
   ✅ All messages acknowledged
```

## Error Handling

The auto_sender handles various error conditions:

- **Network failures**: Graceful handling of connection issues
- **Flow control**: Automatic blocking when send window is full
- **Timeouts**: Configurable ACK timeout handling
- **Rate limiting**: Precise timing control for packet transmission

## Testing

The functionality is tested with:

- Unit tests for packet transmission accuracy
- Integration tests with real UDP receiver
- Rate verification tests
- High-throughput stress tests

Run tests with:
```bash
cargo test auto_sender
```