# Test Node Component

The test-node component is the main executable for the BDSPro Victor system. It provides a configurable node that can act as a data source, bridge, or sink in a distributed processing pipeline.

## Overview

The test-node can be configured to operate in three modes:
- **Source**: Generates and sends data streams
- **Bridge**: Receives data from one channel and forwards to another
- **Sink**: Receives and processes data streams

## Configuration

The test-node uses YAML configuration files to define its behavior. Each configuration file can contain multiple node definitions.

### Configuration Schema

```yaml
- connection: <connection_identifier>
  bind: <bind_address>
  commands:
    - type: StartQuery
      q:
        type: <Source|Bridge|Sink>
        # ... type-specific parameters
    - type: Wait
      millis: <duration_ms>
    - type: StopQuery
      id: <query_id>
```

### Source Node Configuration

```yaml
- connection: localhost:8000
  bind: 0.0.0.0:8000
  commands:
    - type: StartQuery
      q:
        type: Source
        downstream_channel: output_channel
        downstream_connection: localhost:8001
        ingestion_rate_in_milliseconds: 100
        should_be_closed: false
    - type: Wait
      millis: 10000
    - type: StopQuery
      id: 0
```

Parameters:
- `downstream_channel`: Channel name for output data
- `downstream_connection`: Target connection for data
- `ingestion_rate_in_milliseconds`: Rate of data generation
- `should_be_closed`: Whether the channel should be closed after processing

### Bridge Node Configuration

```yaml
- connection: localhost:8001
  bind: 0.0.0.0:8001
  commands:
    - type: StartQuery
      q:
        type: Bridge
        input_channel: input_channel
        downstream_channel: output_channel
        downstream_connection: localhost:8002
        ingestion_rate_in_milliseconds: 50
        should_be_closed: false
    - type: Wait
      millis: 10000
    - type: StopQuery
      id: 0
```

Parameters:
- `input_channel`: Channel name for input data
- `downstream_channel`: Channel name for output data
- `downstream_connection`: Target connection for data
- `ingestion_rate_in_milliseconds`: Rate of data processing
- `should_be_closed`: Whether the channel should be closed after processing

### Sink Node Configuration

```yaml
- connection: localhost:8002
  bind: 0.0.0.0:8002
  commands:
    - type: StartQuery
      q:
        type: Sink
        input_channel: input_channel
        ingestion_rate_in_milliseconds: 20
        expected_messages: 100
        expected_messages_uncertainty: 10
    - type: Wait
      millis: 11000
    - type: StopQuery
      id: 0
```

Parameters:
- `input_channel`: Channel name for input data
- `ingestion_rate_in_milliseconds`: Rate of data processing
- `expected_messages`: Expected number of messages (for validation)
- `expected_messages_uncertainty`: Allowed deviation from expected count

## Usage

### Running a Node

```bash
cargo run --bin test-node -- <config_file.yml> <node_index>
```

Parameters:
- `config_file.yml`: Path to the YAML configuration file
- `node_index`: Index of the node configuration to use (0-based)

### Example: Simple Pipeline

1. Create `pipeline.yml`:
   ```yaml
   - connection: localhost:8000
     bind: 0.0.0.0:8000
     commands:
       - type: StartQuery
         q:
           type: Source
           downstream_channel: data_stream
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
           input_channel: data_stream
           expected_messages: 100
           expected_messages_uncertainty: 10
       - type: Wait
         millis: 11000
       - type: StopQuery
         id: 0
   ```

2. Run the source node:
   ```bash
   cargo run --bin test-node -- pipeline.yml 0
   ```

3. Run the sink node (in another terminal):
   ```bash
   cargo run --bin test-node -- pipeline.yml 1
   ```

## Data Generation

The source node generates test data with the following structure:
- Main buffer: 16 bytes filled with counter values
- Child buffers: Variable number of buffers with different sizes
- Sequence numbers: Incrementing counter for message ordering
- Validation data: Data that can be verified by the sink

## Data Validation

The sink node performs comprehensive validation:
- Sequence number tracking to detect missing messages
- Data integrity verification
- Message count validation
- Performance metrics collection

## Performance Monitoring

The test-node includes built-in performance monitoring:
- Message sequence tracking
- Throughput measurement
- Error detection and reporting
- Resource usage monitoring

## Command Types

### StartQuery
Starts a new query (Source, Bridge, or Sink) with the specified configuration.

### StopQuery
Stops a running query by its ID.

### Wait
Pauses execution for the specified duration in milliseconds.

## Error Handling

The test-node provides comprehensive error handling:
- Network connection failures
- Configuration validation errors
- Data validation failures
- Resource exhaustion detection

## Logging

The test-node uses the `tracing` crate for structured logging:
- Configure log levels with `RUST_LOG` environment variable
- Supports multiple output formats
- Includes performance metrics and debug information

## Examples

See the `tests/` directory for various configuration examples:
- `test.yml`: Basic source-sink pipeline
- `test_nes_bridge.yml`: Multi-node bridge configuration
- `test_ingestion.yml`: Performance testing configuration