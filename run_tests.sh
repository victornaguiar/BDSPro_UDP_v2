#!/bin/bash

# BDSPro Victor Test Runner
# This script helps run various test scenarios

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if cargo is installed
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust and Cargo."
        exit 1
    fi
}

# Build the project
build_project() {
    print_status "Building BDSPro Victor..."
    cargo build --release
    print_status "Build completed successfully!"
}

# Run unit tests
run_unit_tests() {
    print_status "Running unit tests..."
    cargo test
    print_status "Unit tests completed!"
}

# Run a simple source-sink test
run_simple_test() {
    print_status "Running simple source-sink test..."
    
    # Start sink node in background
    cargo run --bin test-node -- test-node/tests/test.yml 1 &
    SINK_PID=$!
    
    # Give sink time to start
    sleep 2
    
    # Start source node in background
    cargo run --bin test-node -- test-node/tests/test.yml 0 &
    SOURCE_PID=$!
    
    # Wait for source to complete
    wait $SOURCE_PID
    
    # Kill sink node
    kill $SINK_PID 2>/dev/null || true
    
    print_status "Simple test completed!"
}

# Run UDP channel test
run_udp_test() {
    print_status "Running UDP channel test..."
    
    # Start receiver in background
    cargo run --bin receiver &
    RECEIVER_PID=$!
    
    # Give receiver time to start
    sleep 2
    
    # Start sender
    timeout 10s cargo run --bin sender || true
    
    # Kill receiver
    kill $RECEIVER_PID 2>/dev/null || true
    
    print_status "UDP test completed!"
}

# Run bridge test
run_bridge_test() {
    print_status "Running bridge test..."
    
    # This requires multiple nodes, so we'll run a simpler version
    print_warning "Bridge test requires manual setup with multiple terminals"
    print_warning "Use: cargo run --bin test-node -- test-node/tests/test_nes_bridge.yml <node_index>"
}

# Run performance test
run_performance_test() {
    print_status "Running performance test..."
    
    # Build in release mode for performance
    cargo build --release
    
    # Start sink node
    ./target/release/test-node test-node/tests/test_ingestion.yml 1 &
    SINK_PID=$!
    
    # Give sink time to start
    sleep 2
    
    # Start source node with high throughput
    timeout 30s ./target/release/test-node test-node/tests/test_ingestion.yml 0 || true
    
    # Kill sink node
    kill $SINK_PID 2>/dev/null || true
    
    print_status "Performance test completed!"
}

# Clean build artifacts
clean_build() {
    print_status "Cleaning build artifacts..."
    cargo clean
    print_status "Clean completed!"
}

# Run all tests
run_all_tests() {
    print_status "Running all tests..."
    build_project
    run_unit_tests
    run_simple_test
    run_udp_test
    run_performance_test
    print_status "All tests completed!"
}

# Show help
show_help() {
    cat << EOF
BDSPro Victor Test Runner

Usage: $0 [COMMAND]

Commands:
    build       Build the project
    test        Run unit tests
    simple      Run simple source-sink test
    udp         Run UDP channel test
    bridge      Run bridge test (requires manual setup)
    perf        Run performance test
    clean       Clean build artifacts
    all         Run all tests
    help        Show this help message

Examples:
    $0 build
    $0 test
    $0 simple
    $0 all

EOF
}

# Main script logic
main() {
    check_cargo
    
    case "${1:-help}" in
        build)
            build_project
            ;;
        test)
            run_unit_tests
            ;;
        simple)
            build_project
            run_simple_test
            ;;
        udp)
            build_project
            run_udp_test
            ;;
        bridge)
            build_project
            run_bridge_test
            ;;
        perf)
            run_performance_test
            ;;
        clean)
            clean_build
            ;;
        all)
            run_all_tests
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"