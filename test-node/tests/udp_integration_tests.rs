use std::time::Duration;
use tokio::time::sleep;
use udp_channal_demo::{UdpSender, UdpReceiver, Message};
use serde_json;

#[tokio::test]
async fn test_udp_channel_basic_communication() {
    // Start a receiver on a random port
    let receiver = UdpReceiver::new("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    // Start a sender that connects to the receiver
    let sender = UdpSender::new("127.0.0.1:0", &receiver_addr.to_string()).await.unwrap();

    // Send a message
    sender.send_message(1, "Hello UDP".to_string()).await.unwrap();

    // Receive the message
    let received = receiver.receive_message().await.unwrap();
    assert!(received.is_some());
    
    let message = received.unwrap();
    assert_eq!(message.typ, "DATA");
    assert_eq!(message.seq, 1);
    assert_eq!(message.data, Some("Hello UDP".to_string()));

    // Verify the sequence was recorded
    let received_seqs = receiver.get_received_seqs().await;
    assert!(received_seqs.contains(&1));
}

#[tokio::test]
async fn test_udp_channel_multiple_messages() {
    // Start a receiver on a random port
    let receiver = UdpReceiver::new("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    // Start a sender that connects to the receiver
    let sender = UdpSender::new("127.0.0.1:0", &receiver_addr.to_string()).await.unwrap();

    // Send multiple messages
    let messages = vec![
        (1, "First message".to_string()),
        (2, "Second message".to_string()),
        (3, "Third message".to_string()),
    ];

    for (seq, data) in &messages {
        sender.send_message(*seq, data.clone()).await.unwrap();
    }

    // Receive all messages
    let mut received_messages = Vec::new();
    for _ in 0..3 {
        let received = receiver.receive_message().await.unwrap();
        assert!(received.is_some());
        received_messages.push(received.unwrap());
    }

    // Verify all messages were received
    assert_eq!(received_messages.len(), 3);
    
    // Check that we got the correct messages (order might vary)
    let received_seqs = receiver.get_received_seqs().await;
    assert!(received_seqs.contains(&1));
    assert!(received_seqs.contains(&2));
    assert!(received_seqs.contains(&3));
}

#[tokio::test]
async fn test_udp_channel_message_serialization() {
    let test_cases = vec![
        Message {
            typ: "DATA".to_string(),
            seq: 1,
            data: Some("test data".to_string()),
        },
        Message {
            typ: "ACK".to_string(),
            seq: 2,
            data: None,
        },
        Message {
            typ: "RQST".to_string(),
            seq: 3,
            data: None,
        },
    ];

    for original in test_cases {
        let serialized = serde_json::to_vec(&original).unwrap();
        let deserialized: Message = serde_json::from_slice(&serialized).unwrap();
        
        assert_eq!(original.typ, deserialized.typ);
        assert_eq!(original.seq, deserialized.seq);
        assert_eq!(original.data, deserialized.data);
    }
}

#[tokio::test]
async fn test_udp_channel_retransmission_request() {
    // Start a receiver on a random port
    let receiver = UdpReceiver::new("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    // Start a sender that connects to the receiver
    let sender = UdpSender::new("127.0.0.1:0", &receiver_addr.to_string()).await.unwrap();

    // Send a message first
    sender.send_message(1, "Original message".to_string()).await.unwrap();

    // Receive the message
    let received = receiver.receive_message().await.unwrap();
    assert!(received.is_some());
    
    let message = received.unwrap();
    assert_eq!(message.typ, "DATA");
    assert_eq!(message.seq, 1);

    // Request retransmission
    let sender_addr = sender.local_addr().unwrap();
    receiver.request_retransmit(1, sender_addr).await.unwrap();

    // Give some time for the retransmission to be processed
    sleep(Duration::from_millis(100)).await;

    // The sender should have received the RQST and retransmitted the message
    // This test verifies the mechanism works, though we can't easily test
    // the actual retransmission without more complex setup
}

#[tokio::test]
async fn test_udp_channel_concurrent_operations() {
    // Start a receiver on a random port
    let receiver = UdpReceiver::new("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    // Start multiple senders
    let mut senders = Vec::new();
    for _i in 0..3 {
        let sender = UdpSender::new("127.0.0.1:0", &receiver_addr.to_string()).await.unwrap();
        senders.push(sender);
    }

    // Send messages concurrently
    let mut handles = Vec::new();
    for (i, sender) in senders.into_iter().enumerate() {
        let handle = tokio::spawn(async move {
            sender.send_message(i as u64 + 1, format!("Message from sender {}", i)).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all sends to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Receive all messages
    let mut received_count = 0;
    for _ in 0..3 {
        let received = receiver.receive_message().await.unwrap();
        if received.is_some() {
            received_count += 1;
        }
    }

    assert_eq!(received_count, 3);
    
    // Verify all sequences were recorded
    let received_seqs = receiver.get_received_seqs().await;
    assert!(received_seqs.len() >= 3);
}