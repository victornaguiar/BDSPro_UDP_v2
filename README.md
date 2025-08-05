# BDSPro Victor - Distributed Data Processing System

**BDSPro Victor** is a distributed data processing system designed for high-performance network-based data streaming and processing. The system consists of multiple interconnected components that work together to provide scalable data processing capabilities.

## Overview

This project implements a distributed data processing framework with the following key components:

- **Test Node**: A configurable node that can act as a data source, sink, or bridge
- **Network Layer**: High-performance network communication using Tokio async runtime
- **Bridge of Bridges**: C++ integration layer for external system interoperability
- **UDP Channel**: External UDP-based communication channel

## Architecture

The system follows a distributed architecture where nodes can be configured to:

1. **Generate Data** (Source): Creates and sends data streams to downstream nodes
2. **Process Data** (Bridge): Receives data from one channel and forwards to another
3. **Consume Data** (Sink): Receives and processes final data streams

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Source    │───▶│   Bridge    │───▶│    Sink     │
│   Node      │    │    Node     │    │    Node     │
└─────────────┘    └─────────────┘    └─────────────┘
```

## Features

- **Asynchronous Processing**: Built on Tokio for high-performance async I/O
- **Configurable Topology**: Define complex data processing pipelines via YAML configuration
- **Network Resilience**: Handles network failures and connection management
- **Performance Monitoring**: Built-in sequence tracking and message verification
- **Cross-Platform**: Works on Linux, macOS, and Windows
- **C++ Integration**: Bridge components for native code integration

## Components

### 1. Test Node (`test-node/`)
The main executable that can be configured to run as different types of nodes:
- **Source Node**: Generates test data at configurable rates
- **Bridge Node**: Forwards data between network channels
- **Sink Node**: Consumes and validates received data

### 2. Network Layer (`network/`)
Provides the core networking functionality:
- Asynchronous TCP/UDP communication
- Protocol serialization/deserialization
- Connection management and pooling
- Message queuing and flow control

### 3. Bridge of Bridges (`bridge-of-bridges/`)
Integration layer for external systems:
- C++ bindings for native code integration
- Distributed bridge networking
- Logging integration via spdlog

### 4. External Components (`external/`)
Additional components for specialized use cases:
- UDP Channel implementation for UDP-based communication

## Getting Started

### Prerequisites

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)
- **C++ Compiler**: Required for bridge components
- **CMake**: For building C++ components

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/victornaguiar/BDSPro_victor.git
   cd BDSPro_victor
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run tests:
   ```bash
   cargo test
   ```

### Basic Usage

#### Simple Source-Sink Example

1. Create a configuration file `simple_config.yml`:
   ```yaml
   - connection: localhost:8000
     bind: 0.0.0.0:8000
     commands:
       - type: StartQuery
         q:
           type: Source
           downstream_channel: data_channel
           downstream_connection: localhost:8001
           ingestion_rate_in_milliseconds: 100
       - type: Wait
         millis: 10000
       - type: StopQuery
         id: 0

   - connection: localhost:8001
     bind: 0.0.0.0:8001
     commands:
       - type: StartQuery
         q:
           type: Sink
           input_channel: data_channel
           expected_messages: 100
           expected_messages_uncertainty: 10
       - type: Wait
         millis: 11000
       - type: StopQuery
         id: 0
   ```

2. Run the source node:
   ```bash
   cargo run --bin test-node -- simple_config.yml 0
   ```

3. In another terminal, run the sink node:
   ```bash
   cargo run --bin test-node -- simple_config.yml 1
   ```

#### UDP Channel Example

1. Start the UDP receiver:
   ```bash
   cargo run --bin receiver
   ```

2. In another terminal, start the UDP sender:
   ```bash
   cargo run --bin sender
   ```

#### Automatic UDP Packet Sending

For automated testing and performance evaluation, use the `auto_sender` binary:

1. Start the UDP receiver:
   ```bash
   cargo run --bin receiver
   ```

2. Send packets automatically for XXX seconds at YYY packets per second:
   ```bash
   # Send 50 packets over 10 seconds (5 packets/sec)
   cargo run --bin auto_sender -- --duration 10 --rate 5 --verbose
   
   # High-rate testing: 100 packets/sec for 30 seconds
   cargo run --bin auto_sender -- --duration 30 --rate 100 --window-size 50
   ```

See [AUTO_SENDER_USAGE.md](AUTO_SENDER_USAGE.md) for detailed usage instructions.

### Configuration

The system uses YAML configuration files to define node behavior. Each node configuration includes:

- **Connection Information**: IP addresses and ports
- **Commands**: Sequence of operations to perform
- **Query Types**: Source, Bridge, or Sink configurations

#### Configuration Schema

```yaml
- connection: <node_address>
  bind: <bind_address>
  commands:
    - type: StartQuery
      q:
        type: <Source|Bridge|Sink>
        # ... type-specific parameters
    - type: Wait
      millis: <duration>
    - type: StopQuery
      id: <query_id>
```

## Development

### Project Structure

```
BDSPro_victor/
├── Cargo.toml              # Workspace configuration
├── bridge-of-bridges/      # C++ integration layer
│   ├── network/           # Distributed bridge networking
│   ├── spdlog/            # Logging integration
│   └── src/               # Main bridge implementation
├── external/              # External components
│   └── BDSPro_UDP_Channel/ # UDP communication
├── network/               # Core networking library
│   └── src/               # Network protocol implementation
├── test-node/             # Main test node executable
│   ├── src/               # Source code
│   └── tests/             # Test configurations
└── README.md              # This file
```

### Building Components

- **All Components**: `cargo build --release`
- **Test Node Only**: `cargo build --release --bin test-node`
- **UDP Channel**: `cargo build --release --bin sender --bin receiver`
- **Run Tests**: `cargo test`

### Adding New Node Types

1. Define the new query type in `test-node/src/config.rs`
2. Implement the execution logic in `test-node/src/main.rs`
3. Add configuration examples in `test-node/tests/`

## Performance Considerations

- **Ingestion Rate**: Configure appropriate rates to avoid overwhelming downstream nodes
- **Buffer Sizes**: Adjust buffer sizes based on expected data volumes
- **Network Topology**: Design topology to minimize network hops
- **Resource Allocation**: Ensure adequate CPU and memory for async processing

## Troubleshooting

### Common Issues

1. **Port Already in Use**: Ensure no other processes are using configured ports
2. **Network Timeouts**: Check firewall settings and network connectivity
3. **Memory Issues**: Monitor memory usage during high-throughput operations
4. **Build Errors**: Ensure all dependencies are correctly installed

### Debugging

- Enable debug logging: `RUST_LOG=debug cargo run ...`
- Use tracing for detailed execution flow
- Check sequence numbers for message ordering issues

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Tokio](https://tokio.rs/) for async I/O
- Uses [serde](https://serde.rs/) for serialization
- Integrates with [spdlog](https://github.com/gabime/spdlog) for logging