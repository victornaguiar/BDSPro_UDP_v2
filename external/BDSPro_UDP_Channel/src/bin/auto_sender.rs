use clap::Parser;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::{interval, sleep};
use udp_channal_demo::{UdpSender, FlowControlConfig};

/// Configuration for automatic UDP sender
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AutoSenderConfig {
    /// Target address to send UDP messages to
    pub target_address: String,
    /// Local bind address (use "127.0.0.1:0" for automatic port)
    pub bind_address: String,
    /// Interval between messages in milliseconds
    pub interval_ms: u64,
    /// Total number of messages to send (0 = infinite)
    pub message_count: u64,
    /// Message template (use {seq} for sequence number, {timestamp} for current time)
    pub message_template: String,
    /// Flow control configuration
    pub flow_control: Option<AutoSenderFlowControl>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AutoSenderFlowControl {
    pub send_window_size: usize,
    pub receive_buffer_size: usize,
    pub ack_timeout_ms: u64,
    pub send_timeout_ms: u64,
}

impl Default for AutoSenderConfig {
    fn default() -> Self {
        Self {
            target_address: "127.0.0.1:8080".to_string(),
            bind_address: "127.0.0.1:0".to_string(),
            interval_ms: 1000,
            message_count: 10,
            message_template: "Automatic message #{seq} sent at {timestamp}".to_string(),
            flow_control: None,
        }
    }
}

impl From<AutoSenderFlowControl> for FlowControlConfig {
    fn from(config: AutoSenderFlowControl) -> Self {
        Self {
            send_window_size: config.send_window_size,
            receive_buffer_size: config.receive_buffer_size,
            ack_timeout: Duration::from_millis(config.ack_timeout_ms),
            send_timeout: Duration::from_millis(config.send_timeout_ms),
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "auto_sender")]
#[command(about = "Automatic UDP package sender with configurable parameters")]
struct Args {
    /// Configuration file path (YAML format)
    #[arg(short, long)]
    config: Option<String>,

    /// Target address (overrides config file)
    #[arg(short, long)]
    target: Option<String>,

    /// Interval between messages in milliseconds (overrides config file)
    #[arg(short, long)]
    interval: Option<u64>,

    /// Number of messages to send, 0 for infinite (overrides config file)
    #[arg(short = 'n', long)]
    count: Option<u64>,

    /// Message template (overrides config file)
    #[arg(short, long)]
    message: Option<String>,

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
        AutoSenderConfig::default()
    };

    // Override with command line arguments
    if let Some(target) = args.target {
        config.target_address = target;
    }
    if let Some(interval) = args.interval {
        config.interval_ms = interval;
    }
    if let Some(count) = args.count {
        config.message_count = count;
    }
    if let Some(message) = args.message {
        config.message_template = message;
    }

    println!("🚀 Starting Automatic UDP Sender");
    println!("   Target: {}", config.target_address);
    println!("   Bind: {}", config.bind_address);
    println!("   Interval: {}ms", config.interval_ms);
    println!("   Count: {} (0 = infinite)", config.message_count);
    println!("   Template: {}", config.message_template);

    // Create UDP sender with optional flow control
    let has_flow_control = config.flow_control.is_some();
    let sender = if let Some(fc_config) = config.flow_control {
        UdpSender::new_with_config(
            &config.bind_address,
            &config.target_address,
            fc_config.into()
        ).await?
    } else {
        UdpSender::new(&config.bind_address, &config.target_address).await?
    };

    println!("✅ UDP Sender connected successfully");
    
    // Set up interval timer
    let sent = if config.interval_ms > 0 {
        let mut timer = interval(Duration::from_millis(config.interval_ms));
        timer.tick().await; // First tick is immediate, skip it

        // Send messages
        let mut sent = 0u64;
        loop {
            // Check if we've reached the message count limit
            if config.message_count > 0 && sent >= config.message_count {
                break;
            }

            // Wait for next interval
            timer.tick().await;

            // Generate message content
            let seq = sent + 1;
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
            let message_content = config.message_template
                .replace("{seq}", &seq.to_string())
                .replace("{timestamp}", &timestamp.to_string());

            // Send message
            match sender.send_message(seq, message_content.clone()).await {
                Ok(_) => {
                    sent += 1;
                    println!("📤 [{}] Sent message: {}", seq, message_content);
                    
                    // Show flow control stats if configured
                    if has_flow_control {
                        let stats = sender.get_flow_control_stats().await;
                        println!("   📊 Flow Control - In-flight: {}, Available: {}/{}", 
                                stats.in_flight_messages, 
                                stats.available_window, 
                                stats.total_window_size);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to send message {}: {}", seq, e);
                    
                    // Add a small delay before retrying
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
        sent
    } else {
        0
    };

    println!("🏁 Finished sending {} messages", sent);
    Ok(())
}

fn load_config(path: &str) -> Result<AutoSenderConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: AutoSenderConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

fn generate_example_config() -> Result<(), Box<dyn std::error::Error>> {
    let example_config = AutoSenderConfig {
        target_address: "127.0.0.1:8080".to_string(),
        bind_address: "127.0.0.1:0".to_string(),
        interval_ms: 1000,
        message_count: 10,
        message_template: "Automatic message #{seq} sent at {timestamp}".to_string(),
        flow_control: Some(AutoSenderFlowControl {
            send_window_size: 10,
            receive_buffer_size: 50,
            ack_timeout_ms: 1000,
            send_timeout_ms: 5000,
        }),
    };

    let yaml_content = serde_yaml::to_string(&example_config)?;
    let filename = "auto_sender_example.yml";
    std::fs::write(filename, yaml_content)?;
    
    println!("📝 Generated example configuration: {}", filename);
    println!("\nTo use the configuration:");
    println!("  cargo run --bin auto_sender -- --config {}", filename);
    println!("\nTo override specific settings:");
    println!("  cargo run --bin auto_sender -- --config {} --target 127.0.0.1:9000 --interval 500", filename);
    
    Ok(())
}