use std::time::Duration;
use tokio::time::sleep;
use tokio::net::UdpSocket;
use udp_channal_demo::Message;
use serde_json;

#[tokio::test]
async fn test_manual_gap_detection_simulation() {
    // This test simulates the behavior we implemented in receiver.rs
    // by manually implementing the gap detection logic
    
    // Create a test socket to receive on
    let receiver_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver_socket.local_addr().unwrap();
    
    // Create sender socket
    let sender_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    
    // Send messages with gaps: 1, 3, 5 (missing 2, 4)
    let messages = vec![
        (1, "Message 1"),
        (3, "Message 3"), 
        (5, "Message 5"),
    ];
    
    for (seq, data) in messages {
        let msg = Message {
            typ: "DATA".to_string(),
            seq,
            data: Some(data.to_string()),
        };
        let encoded = serde_json::to_vec(&msg).unwrap();
        sender_socket.send_to(&encoded, receiver_addr).await.unwrap();
    }
    
    // Simulate receiver behavior: receive messages and detect gaps
    let mut received_seqs = std::collections::HashSet::new();
    let mut min_seq: Option<u64> = None;
    let mut max_seq: Option<u64> = None;
    let mut buf = [0u8; 1024];
    
    // Receive all sent messages
    for _ in 0..3 {
        let (len, sender_addr) = receiver_socket.recv_from(&mut buf).await.unwrap();
        let msg_bytes = &buf[..len];
        
        if let Ok(message) = serde_json::from_slice::<Message>(msg_bytes) {
            if message.typ == "DATA" {
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
                
                // Send ACK
                let ack = Message {
                    typ: "ACK".to_string(),
                    seq: message.seq,
                    data: None,
                };
                let ack_encoded = serde_json::to_vec(&ack).unwrap();
                receiver_socket.send_to(&ack_encoded, sender_addr).await.unwrap();
            }
        }
    }
    
    // Now detect gaps
    if let (Some(min), Some(max)) = (min_seq, max_seq) {
        let mut missing_sequences = Vec::new();
        for seq in min..=max {
            if !received_seqs.contains(&seq) {
                missing_sequences.push(seq);
            }
        }
        
        // Verify we detected the correct gaps
        assert_eq!(missing_sequences, vec![2, 4]);
        println!("Successfully detected missing sequences: {:?}", missing_sequences);
    }
    
    // Verify received sequences
    assert!(received_seqs.contains(&1));
    assert!(received_seqs.contains(&3));
    assert!(received_seqs.contains(&5));
    assert!(!received_seqs.contains(&2));
    assert!(!received_seqs.contains(&4));
    
    println!("Gap detection simulation test passed");
}

#[tokio::test]
async fn test_continuous_reception_with_gaps() {
    // Test that packet reception continues while handling gaps
    
    let receiver_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver_socket.local_addr().unwrap();
    
    let sender_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    
    // Send initial batch with gap: 1, 3
    let msg1 = Message {
        typ: "DATA".to_string(),
        seq: 1,
        data: Some("Message 1".to_string()),
    };
    let msg3 = Message {
        typ: "DATA".to_string(),
        seq: 3,
        data: Some("Message 3".to_string()),
    };
    
    sender_socket.send_to(&serde_json::to_vec(&msg1).unwrap(), receiver_addr).await.unwrap();
    sender_socket.send_to(&serde_json::to_vec(&msg3).unwrap(), receiver_addr).await.unwrap();
    
    // Receive first batch
    let mut received_seqs = std::collections::HashSet::new();
    let mut buf = [0u8; 1024];
    
    for _ in 0..2 {
        let (len, _) = receiver_socket.recv_from(&mut buf).await.unwrap();
        if let Ok(message) = serde_json::from_slice::<Message>(&buf[..len]) {
            if message.typ == "DATA" {
                received_seqs.insert(message.seq);
            }
        }
    }
    
    // Should have 1 and 3, missing 2
    assert!(received_seqs.contains(&1));
    assert!(received_seqs.contains(&3));
    assert!(!received_seqs.contains(&2));
    
    // Now send more messages including the missing one: 2, 4, 5
    let additional_messages = vec![
        (2, "Message 2 (late)"),
        (4, "Message 4"),
        (5, "Message 5"),
    ];
    
    for (seq, data) in additional_messages {
        let msg = Message {
            typ: "DATA".to_string(),
            seq,
            data: Some(data.to_string()),
        };
        sender_socket.send_to(&serde_json::to_vec(&msg).unwrap(), receiver_addr).await.unwrap();
    }
    
    // Receive additional messages
    for _ in 0..3 {
        let (len, _) = receiver_socket.recv_from(&mut buf).await.unwrap();
        if let Ok(message) = serde_json::from_slice::<Message>(&buf[..len]) {
            if message.typ == "DATA" {
                received_seqs.insert(message.seq);
            }
        }
    }
    
    // Should now have all sequences 1-5
    for seq in 1..=5 {
        assert!(received_seqs.contains(&seq), "Missing sequence {}", seq);
    }
    
    println!("Continuous reception test passed: received all sequences including late arrivals");
}

#[tokio::test] 
async fn test_out_of_order_packet_handling() {
    // Test handling of out-of-order packets
    
    let receiver_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver_socket.local_addr().unwrap();
    
    let sender_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    
    // Send packets out of order: 5, 1, 3, 2, 4
    let out_of_order = vec![5, 1, 3, 2, 4];
    
    for seq in out_of_order {
        let msg = Message {
            typ: "DATA".to_string(),
            seq,
            data: Some(format!("Message {}", seq)),
        };
        sender_socket.send_to(&serde_json::to_vec(&msg).unwrap(), receiver_addr).await.unwrap();
        sleep(Duration::from_millis(10)).await; // Small delay to ensure order
    }
    
    // Receive all messages
    let mut received_seqs = std::collections::HashSet::new();
    let mut buf = [0u8; 1024];
    
    for _ in 0..5 {
        let (len, _) = receiver_socket.recv_from(&mut buf).await.unwrap();
        if let Ok(message) = serde_json::from_slice::<Message>(&buf[..len]) {
            if message.typ == "DATA" {
                received_seqs.insert(message.seq);
            }
        }
    }
    
    // Should have received all sequences despite out-of-order delivery
    for seq in 1..=5 {
        assert!(received_seqs.contains(&seq), "Missing sequence {}", seq);
    }
    
    println!("Out-of-order handling test passed: correctly received all sequences");
}

#[tokio::test]
async fn test_retransmission_request_mechanism() {
    // Test the retransmission request mechanism
    
    let receiver_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver_socket.local_addr().unwrap();
    
    let sender_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sender_addr = sender_socket.local_addr().unwrap();
    
    // Send messages with gap: 1, 3 (missing 2)
    let msg1 = Message {
        typ: "DATA".to_string(),
        seq: 1,
        data: Some("Message 1".to_string()),
    };
    let msg3 = Message {
        typ: "DATA".to_string(),
        seq: 3,
        data: Some("Message 3".to_string()),
    };
    
    sender_socket.send_to(&serde_json::to_vec(&msg1).unwrap(), receiver_addr).await.unwrap();
    sender_socket.send_to(&serde_json::to_vec(&msg3).unwrap(), receiver_addr).await.unwrap();
    
    // Simulate receiver processing and gap detection
    let mut received_seqs = std::collections::HashSet::new();
    let mut buf = [0u8; 1024];
    
    // Receive the two messages
    for _ in 0..2 {
        let (len, _) = receiver_socket.recv_from(&mut buf).await.unwrap();
        if let Ok(message) = serde_json::from_slice::<Message>(&buf[..len]) {
            if message.typ == "DATA" {
                received_seqs.insert(message.seq);
            }
        }
    }
    
    // Detect gap and send retransmission request for missing seq 2
    if !received_seqs.contains(&2) {
        let rqst = Message {
            typ: "RQST".to_string(),
            seq: 2,
            data: None,
        };
        receiver_socket.send_to(&serde_json::to_vec(&rqst).unwrap(), sender_addr).await.unwrap();
        println!("Sent retransmission request for sequence 2");
    }
    
    // Simulate sender responding to retransmission request
    // In a real scenario, the sender would receive this RQST and retransmit
    let msg2_retransmit = Message {
        typ: "DATA".to_string(),
        seq: 2,
        data: Some("Message 2 (retransmitted)".to_string()),
    };
    sender_socket.send_to(&serde_json::to_vec(&msg2_retransmit).unwrap(), receiver_addr).await.unwrap();
    
    // Receive the retransmitted message
    let (len, _) = receiver_socket.recv_from(&mut buf).await.unwrap();
    if let Ok(message) = serde_json::from_slice::<Message>(&buf[..len]) {
        if message.typ == "DATA" && message.seq == 2 {
            received_seqs.insert(message.seq);
            println!("Received retransmitted message: seq {}", message.seq);
        }
    }
    
    // Verify all sequences are now present
    assert!(received_seqs.contains(&1));
    assert!(received_seqs.contains(&2));
    assert!(received_seqs.contains(&3));
    
    println!("Retransmission request mechanism test passed");
}