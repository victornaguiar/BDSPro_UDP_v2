# Contributing to BDSPro Victor

Thank you for your interest in contributing to BDSPro Victor! This guide will help you get started with contributing to this project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

Please note that this project is released with a [Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a branch for your changes
4. Make your changes
5. Test your changes
6. Submit a pull request

## Development Setup

### Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **C++ Compiler**: Required for bridge components
- **CMake**: For building C++ components

### Local Development

1. Clone the repository:
   ```bash
   git clone https://github.com/YOUR_USERNAME/BDSPro_victor.git
   cd BDSPro_victor
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Use the test runner for comprehensive testing:
   ```bash
   ./run_tests.sh all
   ```

## Making Changes

### Branch Naming

Use descriptive branch names:
- `feature/add-new-node-type`
- `fix/memory-leak-in-network-layer`
- `docs/improve-readme`
- `refactor/simplify-configuration`

### Commit Messages

Follow the conventional commit format:
- `feat: add new bridge node type`
- `fix: resolve memory leak in network layer`
- `docs: update README with new examples`
- `refactor: simplify configuration loading`
- `test: add integration tests for UDP channel`

### Code Changes

1. **Keep changes focused**: Each PR should address a single issue or feature
2. **Write tests**: Add tests for new functionality
3. **Update documentation**: Update relevant documentation
4. **Follow code style**: Use `cargo fmt` and `cargo clippy`

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific component
cargo test --package test-node

# Run tests with output
cargo test -- --nocapture

# Use the test runner
./run_tests.sh test
```

### Test Types

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflows

### Writing Tests

- Place unit tests in the same file as the code they test
- Place integration tests in the `tests/` directory
- Use descriptive test names
- Test both success and failure cases

Example:
```rust
#[test]
fn test_sequence_tracker_adds_numbers() {
    let mut tracker = MissingSequenceTracker::new();
    tracker.add(1);
    tracker.add(2);
    assert_eq!(tracker.query(), 3);
}
```

## Pull Request Process

1. **Create a clear PR title**: Summarize the changes
2. **Provide detailed description**: Explain what and why
3. **Link related issues**: Reference any related issues
4. **Update documentation**: Include relevant documentation updates
5. **Add tests**: Ensure adequate test coverage
6. **Check CI**: Ensure all checks pass

### PR Template

```markdown
## Description
Brief description of the changes.

## Changes Made
- List of changes
- Another change

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing completed

## Documentation
- [ ] Code comments updated
- [ ] README updated (if necessary)
- [ ] API documentation updated

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Tests pass locally
- [ ] Documentation updated
```

## Code Style

### Rust Code

Follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check for common issues
cargo clippy

# Check for security issues
cargo audit
```

### Code Guidelines

1. **Use meaningful names**: Variables, functions, and types should have clear names
2. **Keep functions small**: Functions should do one thing well
3. **Add documentation**: Use doc comments for public APIs
4. **Handle errors**: Use `Result` types and proper error handling
5. **Avoid unwrap**: Use proper error handling instead of `unwrap()`

### Documentation

- Use doc comments (`///`) for public APIs
- Include examples in doc comments
- Keep comments up-to-date with code changes
- Document complex algorithms and business logic

Example:
```rust
/// Tracks missing sequence numbers in a stream.
///
/// This struct helps identify gaps in sequentially numbered messages,
/// which is useful for detecting packet loss or out-of-order delivery.
///
/// # Examples
///
/// ```
/// let mut tracker = MissingSequenceTracker::new();
/// tracker.add(1);
/// tracker.add(3);
/// assert_eq!(tracker.query(), 2); // 2 is missing
/// ```
pub struct MissingSequenceTracker {
    // ...
}
```

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

1. **Clear description**: What happened vs. what you expected
2. **Steps to reproduce**: Detailed steps to reproduce the issue
3. **Environment info**: OS, Rust version, etc.
4. **Error messages**: Full error messages and stack traces
5. **Configuration**: Relevant configuration files

### Feature Requests

When requesting features:

1. **Use case**: Explain the problem you're trying to solve
2. **Proposed solution**: Suggest how it might work
3. **Alternatives**: Consider alternative approaches
4. **Impact**: Who would benefit from this feature

### Issue Templates

Use the provided issue templates when creating new issues.

## Development Environment

### Recommended Tools

- **IDE**: VS Code with Rust extension
- **Debugger**: Use `rust-gdb` or IDE debugging
- **Profiler**: Use `cargo flamegraph` for performance profiling
- **Documentation**: Use `cargo doc --open` to view docs

### Environment Variables

```bash
# Enable debug logging
export RUST_LOG=debug

# Enable backtraces
export RUST_BACKTRACE=1

# Use faster linker (Linux)
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
```

### Docker Development

```bash
# Build Docker image
docker build -t bdspro-victor .

# Run with Docker Compose
docker-compose up

# Run specific profile
docker-compose --profile udp up
```

## Questions and Support

- **Documentation**: Check the README and component docs
- **Issues**: Search existing issues before creating new ones
- **Discussions**: Use GitHub Discussions for questions
- **Code Review**: Be respectful and constructive in reviews

## Recognition

Contributors will be recognized in the project's acknowledgments section. Thank you for helping make BDSPro Victor better!