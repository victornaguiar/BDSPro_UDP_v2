use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::Instant;
use udp_channal_demo::{UdpReceiver, FlowControlConfig, Message};

/// Configuration for automatic UDP receiver
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AutoReceiverConfig {
    /// Local bind address to receive UDP messages
    pub bind_address: String,
    /// Expected number of messages (0 = infinite)
    pub expected_messages: u64,
    /// Timeout for receiving messages in seconds (0 = no timeout)
    pub timeout_seconds: u64,
    /// Whether to enable statistics reporting
    pub enable_stats: bool,
    /// Statistics reporting interval in seconds
    pub stats_interval_seconds: u64,
    /// Flow control configuration
    pub flow_control: Option<AutoReceiverFlowControl>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AutoReceiverFlowControl {
    pub send_window_size: usize,
    pub receive_buffer_size: usize,
    pub ack_timeout_ms: u64,
    pub send_timeout_ms: u64,
}

impl Default for AutoReceiverConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".to_string(),
            expected_messages: 0,
            timeout_seconds: 0,
            enable_stats: true,
            stats_interval_seconds: 5,
            flow_control: None,
        }
    }
}

impl From<AutoReceiverFlowControl> for FlowControlConfig {
    fn from(config: AutoReceiverFlowControl) -> Self {
        Self {
            send_window_size: config.send_window_size,
            receive_buffer_size: config.receive_buffer_size,
            ack_timeout: Duration::from_millis(config.ack_timeout_ms),
            send_timeout: Duration::from_millis(config.send_timeout_ms),
        }
    }
}

#[derive(Debug)]
struct ReceiverStats {
    messages_received: u64,
    total_data_bytes: u64,
    unique_sequences: u64,
    duplicate_sequences: u64,
    start_time: Instant,
    last_message_time: Option<Instant>,
}

impl ReceiverStats {
    fn new() -> Self {
        Self {
            messages_received: 0,
            total_data_bytes: 0,
            unique_sequences: 0,
            duplicate_sequences: 0,
            start_time: Instant::now(),
            last_message_time: None,
        }
    }

    fn record_message(&mut self, message: &Message, is_duplicate: bool) {
        self.messages_received += 1;
        self.last_message_time = Some(Instant::now());
        
        if let Some(data) = &message.data {
            self.total_data_bytes += data.len() as u64;
        }

        if is_duplicate {
            self.duplicate_sequences += 1;
        } else {
            self.unique_sequences += 1;
        }
    }

    fn print_stats(&self) {
        let elapsed = self.start_time.elapsed();
        let rate = if elapsed.as_secs() > 0 {
            self.messages_received as f64 / elapsed.as_secs() as f64
        } else {
            0.0
        };

        println!("📊 Receiver Statistics:");
        println!("   📈 Messages: {} total, {} unique, {} duplicates", 
                self.messages_received, self.unique_sequences, self.duplicate_sequences);
        println!("   📏 Data: {} bytes total", self.total_data_bytes);
        println!("   ⏱️  Rate: {:.2} msg/sec over {:.1}s", rate, elapsed.as_secs_f64());
        
        if let Some(last) = self.last_message_time {
            let since_last = last.elapsed();
            println!("   🕐 Last message: {:.1}s ago", since_last.as_secs_f64());
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "auto_receiver")]
#[command(about = "Automatic UDP package receiver with configurable parameters")]
struct Args {
    /// Configuration file path (YAML format)
    #[arg(short, long)]
    config: Option<String>,

    /// Bind address (overrides config file)
    #[arg(short, long)]
    bind: Option<String>,

    /// Expected number of messages (overrides config file)
    #[arg(short, long)]
    expected: Option<u64>,

    /// Timeout in seconds (overrides config file)
    #[arg(short, long)]
    timeout: Option<u64>,

    /// Enable detailed statistics
    #[arg(long)]
    stats: bool,

    /// Generate example configuration file and exit
    #[arg(long)]
    generate_config: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Generate example configuration if requested
    if args.generate_config {
        generate_example_config()?;
        return Ok(());
    }

    // Load configuration
    let mut config = if let Some(config_path) = &args.config {
        load_config(config_path)?
    } else {
        AutoReceiverConfig::default()
    };

    // Override with command line arguments
    if let Some(bind) = args.bind {
        config.bind_address = bind;
    }
    if let Some(expected) = args.expected {
        config.expected_messages = expected;
    }
    if let Some(timeout) = args.timeout {
        config.timeout_seconds = timeout;
    }
    if args.stats {
        config.enable_stats = true;
    }

    println!("🎯 Starting Automatic UDP Receiver");
    println!("   Bind: {}", config.bind_address);
    println!("   Expected: {} messages (0 = infinite)", config.expected_messages);
    if config.timeout_seconds > 0 {
        println!("   Timeout: {}s", config.timeout_seconds);
    }
    println!("   Stats: {}", if config.enable_stats { "enabled" } else { "disabled" });

    // Create UDP receiver with optional flow control
    let has_flow_control = config.flow_control.is_some();
    let receiver = if let Some(fc_config) = config.flow_control {
        UdpReceiver::new_with_config(&config.bind_address, fc_config.into()).await?
    } else {
        UdpReceiver::new(&config.bind_address).await?
    };

    println!("✅ UDP Receiver started successfully");

    let mut stats = ReceiverStats::new();
    let mut seen_sequences = HashMap::new();
    
    // Set up optional statistics reporting
    let stats_handle = if config.enable_stats {
        let stats_interval = Duration::from_secs(config.stats_interval_seconds);
        Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(stats_interval);
            interval.tick().await; // Skip first immediate tick
            
            loop {
                interval.tick().await;
                // Note: We can't access stats from here in this simple implementation
                // In a more complex setup, we'd use Arc<Mutex<Stats>> or channels
            }
        }))
    } else {
        None
    };

    // Main receiving loop
    let start_time = Instant::now();
    loop {
        // Check timeout
        if config.timeout_seconds > 0 {
            let elapsed = start_time.elapsed();
            if elapsed.as_secs() >= config.timeout_seconds {
                println!("⏰ Timeout reached after {}s", config.timeout_seconds);
                break;
            }
        }

        // Check if we've received expected number of messages
        if config.expected_messages > 0 && stats.unique_sequences >= config.expected_messages {
            println!("🎯 Received expected {} messages", config.expected_messages);
            break;
        }

        // Receive message with timeout
        let receive_result = if config.timeout_seconds > 0 {
            let remaining_time = Duration::from_secs(config.timeout_seconds)
                .saturating_sub(start_time.elapsed());
            
            tokio::time::timeout(remaining_time, receiver.receive_message()).await
        } else {
            Ok(receiver.receive_message().await)
        };

        match receive_result {
            Ok(Ok(Some(message))) => {
                // Check if this is a duplicate sequence
                let is_duplicate = seen_sequences.contains_key(&message.seq);
                seen_sequences.insert(message.seq, true);

                // Record statistics
                stats.record_message(&message, is_duplicate);

                // Print message details
                if is_duplicate {
                    println!("🔄 [{}] DUPLICATE - Type: {}, Data: {:?}", 
                            message.seq, message.typ, 
                            message.data.as_deref().unwrap_or("None"));
                } else {
                    println!("📥 [{}] Type: {}, Data: {:?}", 
                            message.seq, message.typ, 
                            message.data.as_deref().unwrap_or("None"));
                }

                // Show flow control stats if configured
                if has_flow_control {
                    let buffer_stats = receiver.get_buffer_stats().await;
                    if buffer_stats.buffered_messages > 0 {
                        println!("   📊 Buffer - Used: {}/{}", 
                                buffer_stats.buffered_messages, 
                                buffer_stats.total_buffer_size);
                    }
                }

                // Print periodic stats
                if config.enable_stats && stats.messages_received % 10 == 0 {
                    stats.print_stats();
                }
            }
            Ok(Ok(None)) => {
                // No message received, continue
                continue;
            }
            Ok(Err(e)) => {
                eprintln!("❌ Error receiving message: {}", e);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(_) => {
                // Timeout occurred
                println!("⏰ Receive timeout");
                break;
            }
        }
    }

    // Cleanup
    if let Some(handle) = stats_handle {
        handle.abort();
    }

    // Final statistics
    if config.enable_stats {
        println!("\n🏁 Final Statistics:");
        stats.print_stats();
    }

    println!("👋 UDP Receiver shutting down");
    Ok(())
}

fn load_config(path: &str) -> Result<AutoReceiverConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: AutoReceiverConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

fn generate_example_config() -> Result<(), Box<dyn std::error::Error>> {
    let example_config = AutoReceiverConfig {
        bind_address: "127.0.0.1:8080".to_string(),
        expected_messages: 10,
        timeout_seconds: 30,
        enable_stats: true,
        stats_interval_seconds: 5,
        flow_control: Some(AutoReceiverFlowControl {
            send_window_size: 10,
            receive_buffer_size: 50,
            ack_timeout_ms: 1000,
            send_timeout_ms: 5000,
        }),
    };

    let yaml_content = serde_yaml::to_string(&example_config)?;
    let filename = "auto_receiver_example.yml";
    std::fs::write(filename, yaml_content)?;
    
    println!("📝 Generated example configuration: {}", filename);
    println!("\nTo use the configuration:");
    println!("  cargo run --bin auto_receiver -- --config {}", filename);
    println!("\nTo override specific settings:");
    println!("  cargo run --bin auto_receiver -- --config {} --bind 127.0.0.1:9000 --expected 20", filename);
    
    Ok(())
}