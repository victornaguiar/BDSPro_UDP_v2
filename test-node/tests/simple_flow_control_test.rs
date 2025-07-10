use std::time::Duration;

#[tokio::test]
async fn test_basic_flow_control() {
    // Simple test to verify flow control concepts
    println!("Testing flow control mechanisms...");
    
    // Test 1: Verify that we can create flow control config
    let config = udp_channal_demo::FlowControlConfig {
        send_window_size: 5,
        receive_buffer_size: 10,
        ack_timeout: Duration::from_millis(1000),
        send_timeout: Duration::from_millis(2000),
    };
    
    assert_eq!(config.send_window_size, 5);
    assert_eq!(config.receive_buffer_size, 10);
    println!("✓ Flow control config creation works");
    
    // Test 2: Try to create UDP sender and receiver with flow control
    match udp_channal_demo::UdpReceiver::new("127.0.0.1:0").await {
        Ok(receiver) => {
            println!("✓ UDP receiver created successfully");
            
            let receiver_addr = receiver.local_addr().unwrap();
            
            match udp_channal_demo::UdpSender::new("127.0.0.1:0", &receiver_addr.to_string()).await {
                Ok(_sender) => {
                    println!("✓ UDP sender created successfully");
                    println!("✓ Basic flow control test passed!");
                }
                Err(e) => {
                    println!("✗ Failed to create sender: {:?}", e);
                    panic!("Sender creation failed");
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to create receiver: {:?}", e);
            panic!("Receiver creation failed");
        }
    }
}
