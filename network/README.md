# Network Component

The network component provides the core networking functionality for the BDSPro Victor system. It implements an asynchronous, high-performance network layer using Tokio.

## Overview

This component handles:
- Network protocol definitions
- Connection management
- Message serialization/deserialization
- Channel-based communication
- Flow control and buffering

## Architecture

The network layer is split into two main services:

### Sender Service
- Manages outgoing connections
- Handles message queuing and transmission
- Provides flow control mechanisms
- Supports connection pooling

### Receiver Service
- Listens for incoming connections
- Manages channel registration
- Handles message reception and routing
- Provides backpressure handling

## Protocol

The network protocol uses a tuple-based data structure:

```rust
pub struct TupleBuffer {
    pub sequence_number: u64,
    pub origin_id: u32,
    pub watermark: u64,
    pub chunk_number: u32,
    pub number_of_tuples: u32,
    pub last_chunk: bool,
    pub data: Vec<u8>,
    pub child_buffers: Vec<Vec<u8>>,
}
```

## Usage

### Starting Network Services

```rust
use nes_network::{sender, receiver};

// Start sender service
let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_io()
    .enable_time()
    .build()
    .unwrap();
let sender_service = sender::NetworkService::start(rt, connection_id);

// Start receiver service
let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_io()
    .enable_time()
    .build()
    .unwrap();
let receiver_service = receiver::NetworkService::start(rt, bind_addr, connection_id);
```

### Channel Management

```rust
// Register a sender channel
let control_queue = sender_service.register_channel(
    connection_id,
    channel_id
)?;

// Register a receiver channel
let data_queue = receiver_service.register_channel(channel_id)?;
```

### Sending Data

```rust
use nes_network::sender::ChannelControlMessage;

// Send data through the channel
control_queue.send(ChannelControlMessage::Data(tuple_buffer))?;

// Flush the channel
let (tx, rx) = tokio::sync::oneshot::channel();
control_queue.send(ChannelControlMessage::Flush(tx))?;
rx.await?;

// Terminate the channel
control_queue.send(ChannelControlMessage::Terminate)?;
```

### Receiving Data

```rust
// Receive data from the channel
while let Ok(data) = data_queue.recv().await {
    // Process the received data
    process_data(data);
}
```

## Configuration

The network component uses the following connection identifiers:

- `ConnectionIdentifier`: Identifies remote connections (e.g., "localhost:8080")
- `ChannelIdentifier`: Identifies data channels (e.g., "channel_1")
- `ThisConnectionIdentifier`: Identifies the local connection

## Performance Considerations

- Uses async I/O for high concurrency
- Implements backpressure to prevent memory exhaustion
- Supports connection pooling for efficient resource usage
- Provides flow control mechanisms to handle varying processing speeds

## Error Handling

The network component provides comprehensive error handling:
- Connection failures are automatically retried
- Channel errors are propagated to the application
- Resource cleanup is handled automatically

## Dependencies

- `tokio`: Async runtime and networking
- `tokio-util`: Utilities for Tokio applications
- `serde`: Serialization/deserialization
- `async-channel`: Async channel implementation