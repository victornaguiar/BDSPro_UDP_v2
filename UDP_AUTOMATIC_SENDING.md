# Automatic UDP Package Sending

This document describes how to set up automatic sending of packages through the UDP channel in BDSPro.

## Overview

The BDSPro UDP system now includes automatic UDP package sending capabilities through two new binaries:

- **`auto_sender`**: Automatically sends UDP messages at configurable intervals
- **`auto_receiver`**: Automatically receives and processes UDP messages with statistics

## Quick Start

### 1. Generate Example Configurations

```bash
# Generate sender configuration
cargo run --bin auto_sender -- --generate-config

# Generate receiver configuration  
cargo run --bin auto_receiver -- --generate-config
```

This creates:
- `auto_sender_example.yml` - Configuration for automatic sender
- `auto_receiver_example.yml` - Configuration for automatic receiver

### 2. Start Automatic Communication

```bash
# Terminal 1: Start receiver
cargo run --bin auto_receiver -- --config auto_receiver_example.yml

# Terminal 2: Start sender
cargo run --bin auto_sender -- --config auto_sender_example.yml
```

## Configuration Files

### Sender Configuration (`auto_sender_example.yml`)

```yaml
target_address: 127.0.0.1:8080    # Target UDP address
bind_address: 127.0.0.1:0         # Local bind address (0 = automatic port)
interval_ms: 1000                 # Interval between messages (milliseconds)
message_count: 10                 # Number of messages to send (0 = infinite)
message_template: 'Automatic message #{seq} sent at {timestamp}'
flow_control:                     # Optional flow control settings
  send_window_size: 10
  receive_buffer_size: 50
  ack_timeout_ms: 1000
  send_timeout_ms: 5000
```

### Receiver Configuration (`auto_receiver_example.yml`)

```yaml
bind_address: 127.0.0.1:8080      # Address to bind for receiving
expected_messages: 10             # Expected number of messages (0 = infinite)
timeout_seconds: 30               # Timeout for receiving (0 = no timeout)
enable_stats: true                # Enable statistics reporting
stats_interval_seconds: 5         # Statistics reporting interval
flow_control:                     # Optional flow control settings
  send_window_size: 10
  receive_buffer_size: 50
  ack_timeout_ms: 1000
  send_timeout_ms: 5000
```

## Command Line Usage

### Auto Sender

```bash
# Basic usage with configuration file
cargo run --bin auto_sender -- --config sender_config.yml

# Override specific settings
cargo run --bin auto_sender -- --config sender_config.yml \
  --target 127.0.0.1:9000 \
  --interval 500 \
  --count 20

# Use without configuration file (all defaults/overrides)
cargo run --bin auto_sender -- \
  --target 127.0.0.1:8080 \
  --interval 1000 \
  --count 5 \
  --message "Test message #{seq}"
```

### Auto Receiver

```bash
# Basic usage with configuration file
cargo run --bin auto_receiver -- --config receiver_config.yml

# Override specific settings
cargo run --bin auto_receiver -- --config receiver_config.yml \
  --bind 127.0.0.1:9000 \
  --expected 20 \
  --timeout 60

# Use without configuration file
cargo run --bin auto_receiver -- \
  --bind 127.0.0.1:8080 \
  --expected 10 \
  --stats
```

## Message Template Variables

The sender supports template variables in messages:

- `{seq}` - Sequence number (1, 2, 3, ...)
- `{timestamp}` - Current UTC timestamp

Example:
```yaml
message_template: 'Message #{seq} from sensor at {timestamp}'
```

Produces:
```
Message #1 from sensor at 2024-01-15 10:30:45 UTC
Message #2 from sensor at 2024-01-15 10:30:46 UTC
```

## Flow Control Features

Both sender and receiver support advanced flow control:

### Sender Flow Control
- **Send Window**: Limits number of unacknowledged messages in flight
- **Timeouts**: Automatic retransmission of unacknowledged messages
- **Back-pressure**: Blocks sending when window is full

### Receiver Flow Control
- **Receive Buffer**: Limits number of messages in buffer
- **Acknowledgments**: Automatically sends ACK for received messages
- **Duplicate Detection**: Identifies and reports duplicate messages

## Use Cases

### 1. Sensor Data Simulation
```yaml
# sensor_sender.yml
target_address: 192.168.1.100:8080
interval_ms: 5000  # Every 5 seconds
message_count: 0   # Infinite
message_template: 'SENSOR_DATA:{seq}:{timestamp}:temp=25.4,humidity=60.2'
```

### 2. Load Testing
```yaml
# load_test_sender.yml
target_address: 127.0.0.1:8080
interval_ms: 10    # Very fast - 100 msg/sec
message_count: 1000
message_template: 'LOAD_TEST_{seq}'
```

### 3. Heartbeat Messages
```yaml
# heartbeat_sender.yml
target_address: 127.0.0.1:8080
interval_ms: 30000  # Every 30 seconds
message_count: 0    # Infinite
message_template: 'HEARTBEAT_{timestamp}'
```

## Integration with Test-Node Framework

The automatic UDP functionality can be integrated with the existing test-node framework by:

1. **Using UDP binaries in scripts**: Call the binaries from test scripts
2. **Configuration-based testing**: Create test scenarios using YAML configs
3. **Automated testing**: Use in CI/CD pipelines for UDP functionality testing

### Example Test Script

```bash
#!/bin/bash
# udp_test.sh - Automated UDP testing

echo "Starting UDP automatic test..."

# Start receiver in background
cargo run --bin auto_receiver -- --bind 127.0.0.1:8080 --expected 5 --timeout 10 &
RECEIVER_PID=$!

# Wait for receiver to start
sleep 1

# Send messages
cargo run --bin auto_sender -- --target 127.0.0.1:8080 --count 5 --interval 500

# Wait for receiver to finish
wait $RECEIVER_PID

echo "UDP test completed"
```

## Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```
   Error: Address already in use
   ```
   Solution: Change the port or use `127.0.0.1:0` for automatic port selection

2. **Messages Not Received**
   - Check firewall settings
   - Verify target address matches receiver bind address
   - Ensure receiver is started before sender

3. **Flow Control Errors**
   ```
   Error: SendWindowFull
   ```
   Solution: Increase `send_window_size` or reduce sending rate

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug cargo run --bin auto_sender -- --config config.yml
```

## Performance Considerations

- **Sending Rate**: Higher rates may overwhelm receivers or network
- **Flow Control**: Tune window sizes based on network conditions
- **Message Size**: Larger messages may require UDP fragmentation
- **Buffer Sizes**: Adjust based on expected message volumes

## Advanced Configuration Examples

### High-Throughput Sender
```yaml
target_address: 127.0.0.1:8080
bind_address: 127.0.0.1:0
interval_ms: 1        # 1000 msg/sec
message_count: 10000
message_template: 'DATA_{seq}'
flow_control:
  send_window_size: 100    # Large window
  send_timeout_ms: 10000   # Long timeout
```

### Robust Receiver
```yaml
bind_address: 0.0.0.0:8080     # Accept from any IP
expected_messages: 0           # Infinite
timeout_seconds: 0             # No timeout
enable_stats: true
stats_interval_seconds: 1      # Frequent stats
flow_control:
  receive_buffer_size: 1000    # Large buffer
```

This automatic UDP package sending system provides a flexible, configurable way to test and use UDP communication in the BDSPro framework.