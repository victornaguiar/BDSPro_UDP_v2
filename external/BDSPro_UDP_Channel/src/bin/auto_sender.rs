use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, Instant};
use udp_channal_demo::{UdpSender, FlowControlConfig};

/// Automatic UDP sender that sends packets at a specified rate for a specified duration
#[derive(Parser)]
#[command(name = "auto_sender")]
#[command(about = "Automatically send UDP packets with data integrity features")]
#[command(version = "1.0")]
struct Args {
    /// Target address to send packets to (format: IP:PORT)
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    target: String,

    /// Local bind address (format: IP:PORT)
    #[arg(short, long, default_value = "127.0.0.1:0")]
    bind: String,

    /// Duration to send packets in seconds (XXX seconds)
    #[arg(short, long, default_value = "10")]
    duration: u64,

    /// Packets per second rate (YYY packages per second)
    #[arg(short, long, default_value = "5")]
    rate: u64,

    /// Message content prefix (sequence number will be appended)
    #[arg(short, long, default_value = "Auto message")]
    message: String,

    /// Send window size for flow control
    #[arg(long, default_value = "10")]
    window_size: usize,

    /// Acknowledgment timeout in milliseconds
    #[arg(long, default_value = "1000")]
    ack_timeout: u64,

    /// Print statistics during sending
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🚀 UDP Auto Sender Starting...");
    println!("   Target: {}", args.target);
    println!("   Bind: {}", args.bind);
    println!("   Duration: {} seconds", args.duration);
    println!("   Rate: {} packets/second", args.rate);
    println!("   Message: \"{}\"", args.message);
    println!("   Window Size: {}", args.window_size);

    // Validate rate
    if args.rate == 0 {
        eprintln!("❌ Error: Rate must be greater than 0");
        return Ok(());
    }

    // Calculate interval between packets
    let packet_interval = Duration::from_millis(1000 / args.rate);
    let total_duration = Duration::from_secs(args.duration);

    // Configure flow control
    let flow_config = FlowControlConfig {
        send_window_size: args.window_size,
        receive_buffer_size: 50, // Not used for sender
        ack_timeout: Duration::from_millis(args.ack_timeout),
        send_timeout: Duration::from_millis(5000),
    };

    // Create UDP sender with flow control
    let sender = match UdpSender::new_with_config(&args.bind, &args.target, flow_config).await {
        Ok(sender) => {
            println!("✅ UDP Sender created successfully");
            println!("   Local address: {:?}", sender.local_addr()?);
            Arc::new(sender)
        }
        Err(e) => {
            eprintln!("❌ Failed to create UDP sender: {}", e);
            return Ok(());
        }
    };

    // Start sending packets
    println!("\n📡 Starting packet transmission...");
    let start_time = Instant::now();
    let mut interval = interval(packet_interval);
    let mut packets_sent = 0u64;
    let mut packets_failed = 0u64;

    // Statistics task if verbose
    let stats_handle = if args.verbose {
        Some(tokio::spawn({
            let sender_clone = Arc::clone(&sender);
            async move {
                let mut stats_interval = tokio::time::interval(Duration::from_secs(1));
                loop {
                    stats_interval.tick().await;
                    let stats = sender_clone.get_flow_control_stats().await;
                    println!("📊 Flow Control - In-flight: {}/{}, Available window: {}", 
                             stats.in_flight_messages, 
                             stats.total_window_size,
                             stats.available_window);
                    
                    // Break if we've been running too long (safety check)
                    if start_time.elapsed() > total_duration + Duration::from_secs(5) {
                        break;
                    }
                }
            }
        }))
    } else {
        None
    };

    loop {
        interval.tick().await;
        
        // Check if we've exceeded the duration
        if start_time.elapsed() >= total_duration {
            break;
        }

        // Create message with sequence number
        let message_content = format!("{} #{}", args.message, packets_sent + 1);
        
        // Send message using auto-incrementing sequence
        match sender.send_message_auto_seq(message_content.clone()).await {
            Ok(seq) => {
                packets_sent += 1;
                if args.verbose {
                    println!("📤 Sent packet #{} (seq: {}): \"{}\"", packets_sent, seq, message_content);
                }
            }
            Err(e) => {
                packets_failed += 1;
                if args.verbose {
                    eprintln!("❌ Failed to send packet #{}: {}", packets_sent + 1, e);
                }
            }
        }
    }

    // Stop statistics task
    if let Some(handle) = stats_handle {
        handle.abort();
    }

    // Final statistics
    let elapsed = start_time.elapsed();
    let actual_rate = packets_sent as f64 / elapsed.as_secs_f64();
    
    println!("\n📊 Transmission Complete!");
    println!("   Duration: {:.2} seconds", elapsed.as_secs_f64());
    println!("   Packets sent: {}", packets_sent);
    println!("   Packets failed: {}", packets_failed);
    println!("   Actual rate: {:.2} packets/second", actual_rate);
    println!("   Target rate: {} packets/second", args.rate);
    
    if packets_failed > 0 {
        println!("   ⚠️  {} packets failed to send (likely due to flow control)", packets_failed);
    }

    // Wait a bit for final ACKs
    println!("\n⏳ Waiting for final acknowledgments...");
    tokio::time::sleep(Duration::from_millis(args.ack_timeout + 500)).await;
    
    // Final flow control stats
    let final_stats = sender.get_flow_control_stats().await;
    println!("📊 Final Flow Control Stats:");
    println!("   In-flight messages: {}/{}", final_stats.in_flight_messages, final_stats.total_window_size);
    if final_stats.in_flight_messages > 0 {
        println!("   ⚠️  {} messages still awaiting acknowledgment", final_stats.in_flight_messages);
    } else {
        println!("   ✅ All messages acknowledged");
    }

    println!("\n🎉 Auto sender finished!");
    Ok(())
}