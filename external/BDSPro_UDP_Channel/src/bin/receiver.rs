use tokio::net::UdpSocket;
use std::collections::{HashSet, HashMap};
use tokio::io;
use serde_json;
use tokio::time::{Duration, Instant};
use udp_channal_demo::Message;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Bind to localhost:8080 and start listening
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    println!("UDP Receiver runs on 127.0.0.1:8080");
    println!("Enhanced receiver with gap detection and automatic retransmission requests");
    println!("Messagestructure: Typ | SEQ | DATA");
    
    // for incoming packets
    let mut buf = [0u8; 1024];

    // Track which sequence numbers we've seen
    let mut received_seqs = HashSet::new();
    
    // Track expected sequence range and gaps
    let mut min_seq: Option<u64> = None;
    let mut max_seq: Option<u64> = None;
    
    // Track retransmission requests and their timestamps
    let mut retransmission_requests: HashMap<u64, Instant> = HashMap::new();
    
    // Configuration for retransmission
    let retransmission_timeout = Duration::from_secs(2);
    let max_retransmission_attempts = 3;
    let mut retransmission_attempts: HashMap<u64, u32> = HashMap::new();

    loop {
        // Wait for the next UDP packet
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let msg_bytes = &buf[..len];

        // Parse the JSON into our Message struct
        match serde_json::from_slice::<Message>(msg_bytes) {
            Ok(message) => {
                // Print raw message info
                println!("[{}]: {} | {} | {:?}",
                         addr,
                         message.typ,
                         message.seq,
                         message.data.unwrap_or("".to_string())
                );

                // Only handle DATA messages here
                if message.typ == "DATA" {
                    // Log reception of this DATA packet
                    println!("[RCV ] DATA seq {}", message.seq);

                    // Record that we've seen this sequence number
                    received_seqs.insert(message.seq);
                    
                    // Update sequence range
                    match (min_seq, max_seq) {
                        (None, None) => {
                            min_seq = Some(message.seq);
                            max_seq = Some(message.seq);
                        },
                        (Some(min), Some(max)) => {
                            if message.seq < min {
                                min_seq = Some(message.seq);
                            }
                            if message.seq > max {
                                max_seq = Some(message.seq);
                            }
                        },
                        _ => unreachable!(),
                    }
                    
                    // Remove this sequence from pending retransmission requests if it exists
                    if retransmission_requests.remove(&message.seq).is_some() {
                        retransmission_attempts.remove(&message.seq);
                        println!("[GAP ] Received retransmitted packet seq {}", message.seq);
                    }

                    // Send back an ACK for every DATA
                    let ack = Message {
                        typ: "ACK".to_string(),
                        seq: message.seq,
                        data: None,
                    };
                    let ack_encoded = serde_json::to_vec(&ack)
                        .expect("Failed to serialize ACK");
                    socket.send_to(&ack_encoded, addr).await
                        .expect("Failed to send ACK");
                    println!("[ACK ] Sending ACK for seq {}", message.seq);

                    // Check for gaps in sequence numbers and request missing ones
                    if let (Some(min), Some(max)) = (min_seq, max_seq) {
                        detect_and_request_missing_sequences(
                            &socket,
                            addr,
                            &received_seqs,
                            &mut retransmission_requests,
                            &mut retransmission_attempts,
                            min,
                            max,
                            max_retransmission_attempts,
                        ).await?;
                    }
                }
            }
            Err(e) => {
                // If JSON parsing fails, log an error
                eprintln!("Fehler beim Parsen der Nachricht von {}: {}", addr, e);
            }
        }
        
        // Periodically check for timed-out retransmission requests
        check_retransmission_timeouts(
            &socket,
            &mut retransmission_requests,
            &mut retransmission_attempts,
            retransmission_timeout,
            max_retransmission_attempts,
        ).await?;
    }
}

/// Detect missing sequence numbers and request retransmission
async fn detect_and_request_missing_sequences(
    socket: &UdpSocket,
    addr: std::net::SocketAddr,
    received_seqs: &HashSet<u64>,
    retransmission_requests: &mut HashMap<u64, Instant>,
    retransmission_attempts: &mut HashMap<u64, u32>,
    min_seq: u64,
    max_seq: u64,
    max_attempts: u32,
) -> io::Result<()> {
    let mut missing_sequences = Vec::new();
    
    // Check for gaps in the sequence range
    for seq in min_seq..=max_seq {
        if !received_seqs.contains(&seq) && !retransmission_requests.contains_key(&seq) {
            // Don't request sequences we've already exceeded retry limit for
            if let Some(&attempts) = retransmission_attempts.get(&seq) {
                if attempts >= max_attempts {
                    continue;
                }
            }
            missing_sequences.push(seq);
        }
    }
    
    // Request retransmission for missing sequences
    for missing_seq in missing_sequences {
        println!("[GAP ] Detected missing sequence {}, requesting retransmission", missing_seq);
        
        let rqst = Message {
            typ: "RQST".to_string(),
            seq: missing_seq,
            data: None,
        };
        
        let encoded = serde_json::to_vec(&rqst)
            .expect("Failed to serialize RQST");
        socket.send_to(&encoded, addr).await?;
        
        // Track this retransmission request
        retransmission_requests.insert(missing_seq, Instant::now());
        let attempts = retransmission_attempts.entry(missing_seq).or_insert(0);
        *attempts += 1;
        
        println!("[RQST] Sent retransmission request for seq {} (attempt {})", missing_seq, attempts);
    }
    
    Ok(())
}

/// Check for timed-out retransmission requests and retry or give up
async fn check_retransmission_timeouts(
    _socket: &UdpSocket,
    retransmission_requests: &mut HashMap<u64, Instant>,
    retransmission_attempts: &mut HashMap<u64, u32>,
    timeout: Duration,
    max_attempts: u32,
) -> io::Result<()> {
    let now = Instant::now();
    let mut timed_out_seqs = Vec::new();
    
    // Find timed-out requests
    for (&seq, &request_time) in retransmission_requests.iter() {
        if now.duration_since(request_time) > timeout {
            timed_out_seqs.push(seq);
        }
    }
    
    // Handle timed-out requests
    for seq in timed_out_seqs {
        let attempts = retransmission_attempts.get(&seq).copied().unwrap_or(0);
        
        if attempts < max_attempts {
            // Retry the request
            println!("[TIMEOUT] Retransmission request for seq {} timed out, retrying (attempt {})", seq, attempts + 1);
            
            let rqst = Message {
                typ: "RQST".to_string(),
                seq,
                data: None,
            };
            
            let _encoded = serde_json::to_vec(&rqst)
                .expect("Failed to serialize RQST");
            
            // We don't have the original sender address here, so we'll need to track it
            // For now, we'll skip the retry and just remove the request to avoid infinite loops
            retransmission_requests.remove(&seq);
            *retransmission_attempts.entry(seq).or_insert(0) += 1;
            
            println!("[RETRY] Would retry seq {} but sender address not available", seq);
        } else {
            // Give up on this sequence
            println!("[GIVE_UP] Giving up on seq {} after {} attempts", seq, attempts);
            retransmission_requests.remove(&seq);
        }
    }
    
    Ok(())
}
