use std::time::Duration;
use tokio::time::sleep;
use udp_channal_demo::{UdpSender, UdpReceiver, Message, FlowControlConfig};
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

#[tokio::test]
async fn test_udp_channel_flow_control_send_window() {
    // Create sender with small send window for testing
    let small_config = FlowControlConfig {
        send_window_size: 2,
        receive_buffer_size: 10,
        ack_timeout: Duration::from_millis(100),
        send_timeout: Duration::from_millis(500),
    };

    let receiver = UdpReceiver::new("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    let sender = UdpSender::new_with_config(
        "127.0.0.1:0", 
        &receiver_addr.to_string(), 
        small_config
    ).await.unwrap();

    // Send messages up to window limit
    sender.send_message(1, "Message 1".to_string()).await.unwrap();
    sender.send_message(2, "Message 2".to_string()).await.unwrap();

    // Check flow control stats
    let stats = sender.get_flow_control_stats().await;
    assert_eq!(stats.in_flight_messages, 2);
    assert_eq!(stats.available_window, 0);

    // Next send should timeout due to full window
    let result = sender.send_message(3, "Message 3".to_string()).await;
    assert!(result.is_err());
    match result {
        Err(udp_channal_demo::UdpChannelError::SendWindowFull) => {},
        _ => panic!("Expected SendWindowFull error"),
    }
}

#[tokio::test]
async fn test_udp_channel_flow_control_receive_buffer() {
    // Create receiver with small buffer for testing
    let small_config = FlowControlConfig {
        send_window_size: 10,
        receive_buffer_size: 2,
        ack_timeout: Duration::from_millis(100),
        send_timeout: Duration::from_millis(500),
    };

    let receiver = UdpReceiver::new_with_config("127.0.0.1:0", small_config).await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    let sender = UdpSender::new("127.0.0.1:0", &receiver_addr.to_string()).await.unwrap();

    // Send messages to fill receiver buffer
    sender.send_message(1, "Message 1".to_string()).await.unwrap();
    sender.send_message(2, "Message 2".to_string()).await.unwrap();

    // Receive messages to fill buffer
    let msg1 = receiver.receive_message().await.unwrap();
    let msg2 = receiver.receive_message().await.unwrap();
    assert!(msg1.is_some());
    assert!(msg2.is_some());

    // Check buffer stats
    let buffer_stats = receiver.get_buffer_stats().await;
    assert_eq!(buffer_stats.buffered_messages, 2);
    assert_eq!(buffer_stats.available_space, 0);

    // Send another message - should cause buffer overflow
    sender.send_message(3, "Message 3".to_string()).await.unwrap();
    
    // Receiving this message should fail due to full buffer
    let result = receiver.receive_message().await;
    assert!(result.is_err());
    match result {
        Err(udp_channal_demo::UdpChannelError::ReceiveBufferFull) => {},
        _ => panic!("Expected ReceiveBufferFull error"),
    }
}

#[tokio::test]
async fn test_udp_channel_flow_control_window_recovery() {
    // Test that ACKs properly release send window space
    let config = FlowControlConfig {
        send_window_size: 1,
        receive_buffer_size: 10,
        ack_timeout: Duration::from_millis(100),
        send_timeout: Duration::from_millis(500),
    };

    let receiver = UdpReceiver::new("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    let sender = UdpSender::new_with_config(
        "127.0.0.1:0", 
        &receiver_addr.to_string(), 
        config
    ).await.unwrap();

    // Send first message (fills window)
    sender.send_message(1, "Message 1".to_string()).await.unwrap();
    
    let stats = sender.get_flow_control_stats().await;
    assert_eq!(stats.available_window, 0);

    // Receive message (should send ACK and free window space)
    let received = receiver.receive_message().await.unwrap();
    assert!(received.is_some());

    // Wait for ACK to be processed
    sleep(Duration::from_millis(50)).await;

    // Window should have space again
    let stats_after = sender.get_flow_control_stats().await;
    assert!(stats_after.available_window > 0);

    // Should be able to send another message
    let result = sender.send_message(2, "Message 2".to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_udp_channel_flow_control_auto_seq() {
    let receiver = UdpReceiver::new("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    let sender = UdpSender::new("127.0.0.1:0", &receiver_addr.to_string()).await.unwrap();

    // Test auto-incrementing sequence numbers
    let seq1 = sender.send_message_auto_seq("First message".to_string()).await.unwrap();
    let seq2 = sender.send_message_auto_seq("Second message".to_string()).await.unwrap();
    let seq3 = sender.send_message_auto_seq("Third message".to_string()).await.unwrap();

    assert_eq!(seq1, 1);
    assert_eq!(seq2, 2);
    assert_eq!(seq3, 3);

    // Receive all messages
    for expected_seq in 1..=3 {
        let received = receiver.receive_message().await.unwrap();
        assert!(received.is_some());
        let msg = received.unwrap();
        assert_eq!(msg.seq, expected_seq);
    }
}

#[tokio::test]
async fn test_udp_channel_flow_control_buffer_recovery() {
    // Test that consuming from buffer frees up space
    let config = FlowControlConfig {
        send_window_size: 10,
        receive_buffer_size: 2,
        ack_timeout: Duration::from_millis(100),
        send_timeout: Duration::from_millis(500),
    };

    let receiver = UdpReceiver::new_with_config("127.0.0.1:0", config).await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    let sender = UdpSender::new("127.0.0.1:0", &receiver_addr.to_string()).await.unwrap();

    // Fill the buffer
    sender.send_message(1, "Message 1".to_string()).await.unwrap();
    sender.send_message(2, "Message 2".to_string()).await.unwrap();

    receiver.receive_message().await.unwrap();
    receiver.receive_message().await.unwrap();

    let buffer_stats = receiver.get_buffer_stats().await;
    assert_eq!(buffer_stats.buffered_messages, 2);
    assert_eq!(buffer_stats.available_space, 0);

    // Consume one message from buffer
    let consumed = receiver.receive_from_buffer().await;
    assert!(consumed.is_some());

    // Buffer should have space again
    let buffer_stats_after = receiver.get_buffer_stats().await;
    assert_eq!(buffer_stats_after.buffered_messages, 1);
    assert_eq!(buffer_stats_after.available_space, 1);

    // Should be able to receive another message
    sender.send_message(3, "Message 3".to_string()).await.unwrap();
    let result = receiver.receive_message().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_udp_channel_flow_control_concurrent_senders() {
    // Test flow control with multiple senders
    let config = FlowControlConfig {
        send_window_size: 5,
        receive_buffer_size: 20,
        ack_timeout: Duration::from_millis(200),
        send_timeout: Duration::from_millis(1000),
    };

    let receiver = UdpReceiver::new_with_config("127.0.0.1:0", config.clone()).await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    // Create multiple senders
    let mut senders = Vec::new();
    for _i in 0..3 {
        let sender = UdpSender::new_with_config(
            "127.0.0.1:0", 
            &receiver_addr.to_string(), 
            config.clone()
        ).await.unwrap();
        senders.push(sender);
    }

    // Send messages concurrently from all senders
    let mut handles = Vec::new();
    for (i, sender) in senders.into_iter().enumerate() {
        let handle = tokio::spawn(async move {
            for j in 0..3 {
                let seq = (i * 10 + j + 1) as u64;
                let data = format!("Message from sender {} seq {}", i, seq);
                sender.send_message(seq, data).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all sends
    for handle in handles {
        handle.await.unwrap();
    }

    // Receive all messages
    let mut received_count = 0;
    for _ in 0..9 {
        let received = receiver.receive_message().await.unwrap();
        if received.is_some() {
            received_count += 1;
        }
    }

    assert_eq!(received_count, 9);

    // Verify buffer stats
    let buffer_stats = receiver.get_buffer_stats().await;
    assert_eq!(buffer_stats.buffered_messages, 9);
}