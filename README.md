# BDSPro UDP Channel Utilities

This repository's primary purpose is the UDP channel tools located in:
`external/BDSPro_UDP_Channel/src/bin`

These binaries are used to send, receive, and automatically send UDP packets for testing and benchmarking.

## Binaries (main focus)

- Receiver: https://github.com/victornaguiar/BDSPro_UDP_v2/blob/main/external/BDSPro_UDP_Channel/src/bin/receiver.rs
- Sender: https://github.com/victornaguiar/BDSPro_UDP_v2/blob/main/external/BDSPro_UDP_Channel/src/bin/sender.rs
- Auto Sender: https://github.com/victornaguiar/BDSPro_UDP_v2/blob/main/external/BDSPro_UDP_Channel/src/bin/auto_sender.rs
- Message example/tooling: https://github.com/victornaguiar/BDSPro_UDP_v2/blob/main/external/BDSPro_UDP_Channel/src/bin/message.rs

Browse the folder:
https://github.com/victornaguiar/BDSPro_UDP_v2/tree/main/external/BDSPro_UDP_Channel/src/bin

## Quick start

Prerequisite: Rust toolchain (install via https://rustup.rs/)

Build:
```bash
cargo build --release --bin receiver --bin sender --bin auto_sender
```

Run the UDP receiver:
```bash
cargo run --bin receiver
```

Run a simple UDP sender (in another terminal):
```bash
cargo run --bin sender
```

Automatically send packets (duration, rate, window size, etc. are configurable):
```bash
# Send 50 packets over 10 seconds (5 packets/sec)
cargo run --bin auto_sender -- --duration 10 --rate 5 --verbose

# High-rate testing: 100 packets/sec for 30 seconds with a custom window
cargo run --bin auto_sender -- --duration 30 --rate 100 --window-size 50
```

Tip: Each binary supports `--help` for available flags and options:
```bash
cargo run --bin receiver -- --help
cargo run --bin sender -- --help
cargo run --bin auto_sender -- --help
```

## Notes

- The rest of the workspace may contain supporting or legacy components, but the primary usage is centered on the UDP channel binaries above.
- If you are only interested in sending/receiving UDP packets for tests or demos, focus on the `external/BDSPro_UDP_Channel/src/bin` folder.

## License

This project is licensed under the MIT License. See the LICENSE file for details.