use tokio::net::UdpSocket;
use std::collections::HashSet;
use tokio::io;
use serde_json;
use rand::Rng;         // for random number generation
use udp_channal_demo::Message;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Bind to localhost:8080 and start listening
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    println!("UDP Receiver runs on 127.0.0.1:8080");
    println!("Messagestructure: Typ | SEQ | DATA");
    // for incoming packets
    let mut buf = [0u8; 1024];

    // Track which sequence numbers we've seen
    let mut received_seqs = HashSet::new();

    // (Declared but not used) could be used for strict in-order delivery
    let mut expected_seq = 1u64;

    // Simple counter to trigger a re-request every 3 messages
    let mut msg_count = 0;

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

                    // Increment our demo counter
                    msg_count += 1;

                    // → Send back an ACK for every DATA
                    let ack = Message {
                        typ: "ACK".to_string(),
                        seq: message.seq,
                        data: None,
                    };
                    let ack_encoded = serde_json::to_vec(&ack)
                        .expect("Failed to serialize ACK");
                    socket.send_to(&ack_encoded, addr).await
                        .expect("Failed to send ACK");
                    println!("[ACK   ] Sending ACK for seq {}", message.seq);

                    // Every 3rd DATA message, pick a random one to re-request
                    if msg_count % 3 == 0 && !received_seqs.is_empty() {
                        // Create a thread-local RNG
                        let mut rng = rand::rng(); // note: should be thread_rng()
                        // Pick a random previously seen sequence
                        let &random_seq = received_seqs
                            .iter()
                            .nth(rng.gen_range(0..received_seqs.len()))
                            .unwrap();

                        // Log that we're requesting a retransmit
                        println!("[TEST] Request SEQ {} again (random)", random_seq);

                        // Build the RQST message
                        let rqst = Message {
                            typ: "RQST".to_string(),
                            seq: random_seq,
                            data: None,
                        };

                        // Serialize and send the RQST
                        let encoded = serde_json::to_vec(&rqst)
                            .expect("Serialisierung fehlgeschlagen");
                        socket.send_to(&encoded, addr).await?;

                        // (Here you attempted to send ACKs as well; 
                        //  this reuses the same socket send_to but no ack variable exists)
                        println!("[SACK ] sent ACK seq {}", message.seq);

                        let ack_bytes = serde_json::to_vec(&ack).unwrap();
                        socket.send_to(&ack_bytes, addr).await?;
                        println!("[SACK ] sent ACK seq {}", message.seq);
                    }
                }
            }
            Err(e) => {
                // If JSON parsing fails, log an error
                eprintln!("Fehler beim Parsen der Nachricht von {}: {}", addr, e);
            }
        }
    }
}
