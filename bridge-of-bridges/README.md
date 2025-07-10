# Bridge of Bridges Component

The bridge-of-bridges component provides a C++ integration layer for the BDSPro Victor system. It enables interoperability between the Rust-based core system and external C++ applications.

## Overview

This component consists of:
- **Distributed Bridge**: Network bridge functionality
- **SPDLog Bridge**: Logging integration with spdlog
- **C++ Bindings**: Foreign Function Interface (FFI) for C++ integration

## Architecture

The bridge-of-bridges is structured as a static library that can be linked with C++ applications:

```
bridge-of-bridges/
├── Cargo.toml           # Main crate configuration
├── network/             # Distributed bridge networking
├── spdlog/              # SPDLog integration
└── src/                 # Main bridge implementation
    └── lib.rs           # Public API exports
```

## Components

### Distributed Bridge (`network/`)

Provides network bridge functionality for distributed processing:
- Cross-language network communication
- Protocol translation between Rust and C++
- Connection management for external systems

### SPDLog Bridge (`spdlog/`)

Integrates with the popular C++ logging library SPDLog:
- Forwards Rust log messages to SPDLog
- Provides unified logging across language boundaries
- Supports various log levels and formats

## Building

The bridge-of-bridges component is built as a static library:

```bash
# Build the bridge component
cargo build --release

# The static library will be available at:
# target/release/libbridge_of_bridges.a
```

## C++ Integration

### Including the Bridge

To use the bridge in a C++ project:

1. Add the static library to your build system
2. Include the generated header files
3. Link against the bridge library

### CMake Example

```cmake
# Find the bridge library
find_library(BRIDGE_LIB
    NAMES bridge_of_bridges
    PATHS ${CMAKE_CURRENT_SOURCE_DIR}/target/release
)

# Add to your target
target_link_libraries(your_target ${BRIDGE_LIB})
```

### Usage in C++

```cpp
#include "bridge_of_bridges.h"

int main() {
    // Initialize the bridge
    init_bridge();
    
    // Use bridge functionality
    // ... your code here ...
    
    // Cleanup
    cleanup_bridge();
    
    return 0;
}
```

## Configuration

The bridge component uses the CXX crate for C++ interoperability:

```toml
[dependencies]
cxx = "1.0"
distributed_bridge = { path = "network" }
spdlog_bridge = { path = "spdlog" }
```

The library is configured to build as a static library:

```toml
[lib]
crate-type = ["staticlib"]
```

## API Reference

### Core Functions

The bridge provides the following core functions:

- `init_bridge()`: Initialize the bridge system
- `cleanup_bridge()`: Clean up resources
- `send_data()`: Send data through the bridge
- `receive_data()`: Receive data from the bridge
- `configure_logging()`: Configure logging settings

### Network Functions

- `create_connection()`: Create a new network connection
- `close_connection()`: Close a network connection
- `send_message()`: Send a message through the network
- `receive_message()`: Receive a message from the network

### Logging Functions

- `log_info()`: Log an info message
- `log_warning()`: Log a warning message
- `log_error()`: Log an error message
- `log_debug()`: Log a debug message

## Error Handling

The bridge provides comprehensive error handling:
- All functions return error codes
- Detailed error messages are available
- Resource cleanup is automatic on failure

## Performance Considerations

- The bridge uses zero-copy data transfer where possible
- Memory management is handled automatically
- Thread-safe operations for multi-threaded C++ applications

## Dependencies

### Rust Dependencies
- `cxx`: C++ interoperability
- `distributed_bridge`: Network functionality
- `spdlog_bridge`: Logging integration

### C++ Dependencies
- Modern C++ compiler (C++17 or later)
- CMake (for building)
- SPDLog library (for logging features)

## Examples

### Basic Usage

```cpp
#include "bridge_of_bridges.h"
#include <iostream>

int main() {
    // Initialize
    if (init_bridge() != 0) {
        std::cerr << "Failed to initialize bridge" << std::endl;
        return 1;
    }
    
    // Send some data
    const char* data = "Hello from C++";
    if (send_data(data, strlen(data)) != 0) {
        std::cerr << "Failed to send data" << std::endl;
        cleanup_bridge();
        return 1;
    }
    
    // Receive response
    char buffer[1024];
    size_t received = 0;
    if (receive_data(buffer, sizeof(buffer), &received) == 0) {
        std::cout << "Received: " << std::string(buffer, received) << std::endl;
    }
    
    // Cleanup
    cleanup_bridge();
    return 0;
}
```

### Logging Example

```cpp
#include "bridge_of_bridges.h"

int main() {
    init_bridge();
    
    // Configure logging
    configure_logging("application.log", LOG_LEVEL_INFO);
    
    // Log messages
    log_info("Application started");
    log_warning("This is a warning");
    log_error("This is an error");
    
    cleanup_bridge();
    return 0;
}
```

## Building with Your Project

### Using CMake

```cmake
cmake_minimum_required(VERSION 3.10)
project(YourProject)

set(CMAKE_CXX_STANDARD 17)

# Find the bridge library
find_library(BRIDGE_LIB
    NAMES bridge_of_bridges
    PATHS ${CMAKE_CURRENT_SOURCE_DIR}/lib
)

# Add your executable
add_executable(your_app main.cpp)

# Link with the bridge
target_link_libraries(your_app ${BRIDGE_LIB})
```

### Using Make

```make
CXX = g++
CXXFLAGS = -std=c++17 -Wall -Wextra
LDFLAGS = -L./lib -lbridge_of_bridges

your_app: main.cpp
	$(CXX) $(CXXFLAGS) -o $@ $< $(LDFLAGS)

.PHONY: clean
clean:
	rm -f your_app
```

## Troubleshooting

### Common Issues

1. **Linking Errors**: Ensure the static library is in your library path
2. **Symbol Not Found**: Check that all required symbols are exported
3. **Runtime Errors**: Verify that `init_bridge()` is called before other functions
4. **Memory Issues**: Always call `cleanup_bridge()` before program exit

### Debug Build

For debugging, build with debug information:

```bash
cargo build --lib
```

This will include debug symbols and additional error checking.