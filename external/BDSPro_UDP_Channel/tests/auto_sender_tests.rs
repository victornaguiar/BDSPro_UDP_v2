use std::sync::atomic::AtomicUsize;
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use udp_channal_demo::{UdpReceiver, Message};

#[tokio::test]
async fn test_auto_sender_functionality() {
    // Start a receiver
    let receiver = UdpReceiver::new("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();
    let receiver_port = receiver_addr.port();

    // Count received messages
    let received_messages = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<Message>::new()));
    let received_clone = received_messages.clone();

    // Spawn receiver task
    let receiver_handle = tokio::spawn(async move {
        for _ in 0..10 {  // Expect about 10 messages (5 seconds * 2 packets/sec)
            if let Ok(Some(msg)) = receiver.receive_message().await {
                received_clone.lock().await.push(msg);
            }
        }
    });

    // Give receiver time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Run auto_sender with specific parameters
    let output = TokioCommand::new("cargo")
        .args(&[
            "run", 
            "--bin", "auto_sender", 
            "--", 
            "--target", &format!("127.0.0.1:{}", receiver_port),
            "--duration", "3",  // 3 seconds
            "--rate", "3",      // 3 packets per second  
            "--message", "Test packet"
        ])
        .output()
        .await
        .expect("Failed to run auto_sender");

    // Wait for receiver to finish
    let _ = tokio::time::timeout(Duration::from_secs(10), receiver_handle).await;

    // Check that auto_sender completed successfully
    assert!(output.status.success(), "auto_sender failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify we received the expected messages
    let messages = received_messages.lock().await;
    assert!(messages.len() >= 7, "Expected at least 7 messages (allowing for timing), got {}", messages.len());
    assert!(messages.len() <= 12, "Expected at most 12 messages (allowing for timing), got {}", messages.len());

    // Verify message content and sequence
    for (i, msg) in messages.iter().enumerate() {
        assert_eq!(msg.typ, "DATA");
        assert!(msg.data.as_ref().unwrap().starts_with("Test packet"));
        // Sequence numbers should be consecutive starting from 1
        assert_eq!(msg.seq, (i + 1) as u64);
    }

    println!("✅ Auto sender test passed! Received {} messages", messages.len());
}

#[tokio::test]
async fn test_auto_sender_high_rate() {
    // Test higher rate sending
    let receiver = UdpReceiver::new("127.0.0.1:0").await.unwrap();
    let receiver_addr = receiver.local_addr().unwrap();
    let receiver_port = receiver_addr.port();

    let received_count = std::sync::Arc::new(AtomicUsize::new(0));
    let count_clone = received_count.clone();

    // Spawn receiver task that counts messages
    let receiver_handle = tokio::spawn(async move {
        for _ in 0..50 {  // Expect many messages
            if let Ok(Some(_)) = receiver.receive_message().await {
                count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            } else {
                // Small delay if no message received
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    });

    // Give receiver time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Run auto_sender with high rate
    let output = TokioCommand::new("cargo")
        .args(&[
            "run", 
            "--bin", "auto_sender", 
            "--", 
            "--target", &format!("127.0.0.1:{}", receiver_port),
            "--duration", "2",  // 2 seconds
            "--rate", "10",     // 10 packets per second
            "--window-size", "20",  // Larger window for high rate
        ])
        .output()
        .await
        .expect("Failed to run auto_sender");

    // Wait for receiver to finish
    let _ = tokio::time::timeout(Duration::from_secs(5), receiver_handle).await;

    // Check that auto_sender completed successfully
    assert!(output.status.success(), "auto_sender failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify we received approximately the right number of messages
    let count = received_count.load(std::sync::atomic::Ordering::Relaxed);
    assert!(count >= 15, "Expected at least 15 messages (allowing for timing), got {}", count);
    assert!(count <= 25, "Expected at most 25 messages (allowing for timing), got {}", count);

    println!("✅ High rate test passed! Received {} messages", count);
}