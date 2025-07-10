use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::time::{Duration, Instant};

pub use message::Message;

pub mod message;

/// Error type for UDP channel operations
#[derive(Debug, thiserror::Error)]
pub enum UdpChannelError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Channel closed")]
    ChannelClosed,
    #[error("Flow control: Send window full")]
    SendWindowFull,
    #[error("Flow control: Receive buffer full")]
    ReceiveBufferFull,
}

/// Flow control configuration
#[derive(Clone, Debug)]
pub struct FlowControlConfig {
    /// Maximum number of unacknowledged messages in flight
    pub send_window_size: usize,
    /// Maximum number of messages in receive buffer
    pub receive_buffer_size: usize,
    /// Timeout for waiting for acknowledgments
    pub ack_timeout: Duration,
    /// Maximum time to wait for send window space
    pub send_timeout: Duration,
}

impl Default for FlowControlConfig {
    fn default() -> Self {
        Self {
            send_window_size: 10,
            receive_buffer_size: 50,
            ack_timeout: Duration::from_millis(1000),
            send_timeout: Duration::from_millis(5000),
        }
    }
}

/// Structure to track in-flight messages for flow control
#[derive(Debug, Clone)]
struct InFlightMessage {
    seq: u64,
    data: String,
    sent_time: Instant,
    retransmit_count: u32,
}

/// Flow control statistics
#[derive(Debug, Clone)]
pub struct FlowControlStats {
    pub in_flight_messages: usize,
    pub available_window: usize,
    pub total_window_size: usize,
}

/// Buffer statistics
#[derive(Debug, Clone)]
pub struct BufferStats {
    pub buffered_messages: usize,
    pub available_space: usize,
    pub total_buffer_size: usize,
}

pub type Result<T> = std::result::Result<T, UdpChannelError>;

/// UDP sender that handles message transmission and retransmission with flow control
pub struct UdpSender {
    socket: Arc<UdpSocket>,
    sent_messages: Arc<Mutex<HashMap<u64, String>>>,
    flow_control: FlowControlConfig,
    send_window: Arc<Semaphore>,
    in_flight: Arc<Mutex<HashMap<u64, InFlightMessage>>>,
    next_seq: Arc<Mutex<u64>>,
}

impl UdpSender {
    /// Create a new UDP sender with default flow control configuration
    pub async fn new(bind_addr: &str, connect_addr: &str) -> Result<Self> {
        Self::new_with_config(bind_addr, connect_addr, FlowControlConfig::default()).await
    }

    /// Create a new UDP sender with custom flow control configuration
    pub async fn new_with_config(bind_addr: &str, connect_addr: &str, config: FlowControlConfig) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        socket.connect(connect_addr).await?;
        let socket = Arc::new(socket);
        let sent_messages = Arc::new(Mutex::new(HashMap::<u64, String>::new()));
        let send_window = Arc::new(Semaphore::new(config.send_window_size));
        let in_flight = Arc::new(Mutex::new(HashMap::<u64, InFlightMessage>::new()));
        let next_seq = Arc::new(Mutex::new(1));

        // Start background task to handle retransmission requests and ACKs
        let recv_socket = socket.clone();
        let resend_map = sent_messages.clone();
        let in_flight_clone = in_flight.clone();
        let send_window_clone = send_window.clone();
        let (tx, mut rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                if let Ok((len, _)) = recv_socket.recv_from(&mut buf).await {
                    if let Ok(message) = serde_json::from_slice::<Message>(&buf[..len]) {
                        match message.typ.as_str() {
                            "RQST" => {
                                let _ = tx.send(("RQST".to_string(), message.seq)).await;
                            }
                            "ACK" => {
                                let _ = tx.send(("ACK".to_string(), message.seq)).await;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        let send_socket = socket.clone();
        tokio::spawn(async move {
            while let Some((msg_type, seq)) = rx.recv().await {
                match msg_type.as_str() {
                    "RQST" => {
                        // Handle retransmission request
                        let map_guard = resend_map.lock().await;
                        if let Some(text) = map_guard.get(&seq) {
                            let msg = Message {
                                typ: "DATA".to_string(),
                                seq,
                                data: Some(text.clone()),
                            };
                            if let Ok(encoded) = serde_json::to_vec(&msg) {
                                let _ = send_socket.send(&encoded).await;
                            }
                        }
                    }
                    "ACK" => {
                        // Handle acknowledgment - remove from in-flight and release window space
                        let mut in_flight_guard = in_flight_clone.lock().await;
                        if in_flight_guard.remove(&seq).is_some() {
                            send_window_clone.add_permits(1);
                        }
                    }
                    _ => {}
                }
            }
        });

        // Start timeout monitoring task
        let timeout_in_flight = in_flight.clone();
        let timeout_socket = socket.clone();
        let timeout_config = config.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                let now = Instant::now();
                let mut in_flight_guard = timeout_in_flight.lock().await;
                let mut to_retransmit = Vec::new();

                for (seq, msg) in in_flight_guard.iter_mut() {
                    if now.duration_since(msg.sent_time) > timeout_config.ack_timeout {
                        msg.sent_time = now;
                        msg.retransmit_count += 1;
                        to_retransmit.push((*seq, msg.data.clone()));
                    }
                }

                drop(in_flight_guard);

                // Retransmit timed-out messages
                for (seq, data) in to_retransmit {
                    let msg = Message {
                        typ: "DATA".to_string(),
                        seq,
                        data: Some(data),
                    };
                    if let Ok(encoded) = serde_json::to_vec(&msg) {
                        let _ = timeout_socket.send(&encoded).await;
                    }
                }
            }
        });

        Ok(Self {
            socket,
            sent_messages,
            flow_control: config,
            send_window,
            in_flight,
            next_seq,
        })
    }

    /// Send a message with flow control - blocks if send window is full
    pub async fn send_message(&self, seq: u64, data: String) -> Result<()> {
        // Try to acquire send window permit with timeout
        let permit = tokio::time::timeout(
            self.flow_control.send_timeout,
            self.send_window.acquire()
        ).await;

        let _permit = match permit {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Err(UdpChannelError::ChannelClosed),
            Err(_) => return Err(UdpChannelError::SendWindowFull),
        };

        let msg = Message {
            typ: "DATA".to_string(),
            seq,
            data: Some(data.clone()),
        };

        // Store in sent messages for potential retransmission
        let mut map = self.sent_messages.lock().await;
        map.insert(seq, data.clone());
        drop(map);

        // Store in in-flight for flow control tracking
        let in_flight_msg = InFlightMessage {
            seq,
            data: data.clone(),
            sent_time: Instant::now(),
            retransmit_count: 0,
        };
        let mut in_flight = self.in_flight.lock().await;
        in_flight.insert(seq, in_flight_msg);
        drop(in_flight);

        let encoded = serde_json::to_vec(&msg)?;
        self.socket.send(&encoded).await?;
        
        // Permit is automatically forgotten here, keeping the window slot occupied
        std::mem::forget(_permit);
        
        Ok(())
    }

    /// Send a message with auto-incrementing sequence number
    pub async fn send_message_auto_seq(&self, data: String) -> Result<u64> {
        let mut seq_guard = self.next_seq.lock().await;
        let seq = *seq_guard;
        *seq_guard += 1;
        drop(seq_guard);

        self.send_message(seq, data).await?;
        Ok(seq)
    }

    /// Get current flow control statistics
    pub async fn get_flow_control_stats(&self) -> FlowControlStats {
        let in_flight_count = self.in_flight.lock().await.len();
        let available_permits = self.send_window.available_permits();
        
        FlowControlStats {
            in_flight_messages: in_flight_count,
            available_window: available_permits,
            total_window_size: self.flow_control.send_window_size,
        }
    }

    /// Get the local socket address
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        self.socket.local_addr().map_err(UdpChannelError::from)
    }
}

/// UDP receiver that handles message reception and acknowledgment with flow control
pub struct UdpReceiver {
    socket: Arc<UdpSocket>,
    received_seqs: Arc<Mutex<HashSet<u64>>>,
    flow_control: FlowControlConfig,
    receive_buffer: Arc<Mutex<VecDeque<Message>>>,
    buffer_semaphore: Arc<Semaphore>,
}

impl UdpReceiver {
    /// Create a new UDP receiver with default flow control configuration
    pub async fn new(bind_addr: &str) -> Result<Self> {
        Self::new_with_config(bind_addr, FlowControlConfig::default()).await
    }

    /// Create a new UDP receiver with custom flow control configuration
    pub async fn new_with_config(bind_addr: &str, config: FlowControlConfig) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        let socket = Arc::new(socket);
        let received_seqs = Arc::new(Mutex::new(HashSet::new()));
        let receive_buffer = Arc::new(Mutex::new(VecDeque::new()));
        let buffer_semaphore = Arc::new(Semaphore::new(config.receive_buffer_size));

        Ok(Self {
            socket,
            received_seqs,
            flow_control: config,
            receive_buffer,
            buffer_semaphore,
        })
    }

    /// Receive and process messages with flow control
    pub async fn receive_message(&self) -> Result<Option<Message>> {
        let mut buf = [0u8; 1024];
        let (len, addr) = self.socket.recv_from(&mut buf).await?;
        let msg_bytes = &buf[..len];

        match serde_json::from_slice::<Message>(msg_bytes) {
            Ok(message) => {
                if message.typ == "DATA" {
                    // Check if receive buffer has space
                    let permit = self.buffer_semaphore.try_acquire();
                    if permit.is_err() {
                        // Buffer is full - send flow control message (optional)
                        let flow_control_msg = Message {
                            typ: "FLOW_CONTROL".to_string(),
                            seq: 0,
                            data: Some("BUFFER_FULL".to_string()),
                        };
                        let fc_encoded = serde_json::to_vec(&flow_control_msg)?;
                        let _ = self.socket.send_to(&fc_encoded, addr).await;
                        return Err(UdpChannelError::ReceiveBufferFull);
                    }

                    // Record that we've seen this sequence number
                    let mut received_seqs = self.received_seqs.lock().await;
                    received_seqs.insert(message.seq);
                    drop(received_seqs);

                    // Add to receive buffer
                    let mut buffer = self.receive_buffer.lock().await;
                    buffer.push_back(message.clone());
                    drop(buffer);

                    // Send back an ACK
                    let ack = Message {
                        typ: "ACK".to_string(),
                        seq: message.seq,
                        data: None,
                    };
                    let ack_encoded = serde_json::to_vec(&ack)?;
                    self.socket.send_to(&ack_encoded, addr).await?;

                    // Keep the permit to maintain buffer space usage
                    std::mem::forget(permit.unwrap());

                    Ok(Some(message))
                } else {
                    Ok(Some(message))
                }
            }
            Err(e) => Err(UdpChannelError::Serialization(e)),
        }
    }

    /// Receive a message from the buffer (non-blocking)
    pub async fn receive_from_buffer(&self) -> Option<Message> {
        let mut buffer = self.receive_buffer.lock().await;
        if let Some(msg) = buffer.pop_front() {
            // Release buffer space
            self.buffer_semaphore.add_permits(1);
            Some(msg)
        } else {
            None
        }
    }

    /// Get current buffer statistics
    pub async fn get_buffer_stats(&self) -> BufferStats {
        let buffer_size = self.receive_buffer.lock().await.len();
        let available_space = self.buffer_semaphore.available_permits();
        
        BufferStats {
            buffered_messages: buffer_size,
            available_space,
            total_buffer_size: self.flow_control.receive_buffer_size,
        }
    }

    /// Send a retransmission request for a specific sequence number
    pub async fn request_retransmit(&self, seq: u64, addr: std::net::SocketAddr) -> Result<()> {
        let rqst = Message {
            typ: "RQST".to_string(),
            seq,
            data: None,
        };

        let encoded = serde_json::to_vec(&rqst)?;
        self.socket.send_to(&encoded, addr).await?;
        Ok(())
    }

    /// Get the list of received sequence numbers
    pub async fn get_received_seqs(&self) -> HashSet<u64> {
        self.received_seqs.lock().await.clone()
    }

    /// Get the local socket address
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        self.socket.local_addr().map_err(UdpChannelError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_udp_sender_creation() {
        let result = UdpSender::new("127.0.0.1:0", "127.0.0.1:8080").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_udp_receiver_creation() {
        let result = UdpReceiver::new("127.0.0.1:0").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_message_serialization() {
        let msg = Message {
            typ: "DATA".to_string(),
            seq: 1,
            data: Some("test data".to_string()),
        };

        let serialized = serde_json::to_vec(&msg).unwrap();
        let deserialized: Message = serde_json::from_slice(&serialized).unwrap();

        assert_eq!(msg.typ, deserialized.typ);
        assert_eq!(msg.seq, deserialized.seq);
        assert_eq!(msg.data, deserialized.data);
    }

    #[tokio::test]
    async fn test_sender_receiver_integration() {
        // Start receiver
        let receiver = UdpReceiver::new("127.0.0.1:0").await.unwrap();
        let receiver_addr = receiver.socket.local_addr().unwrap();

        // Start sender
        let sender = UdpSender::new("127.0.0.1:0", &receiver_addr.to_string()).await.unwrap();

        // Send a message
        sender.send_message(1, "test message".to_string()).await.unwrap();

        // Receive the message
        let received = receiver.receive_message().await.unwrap();
        assert!(received.is_some());
        let msg = received.unwrap();
        assert_eq!(msg.typ, "DATA");
        assert_eq!(msg.seq, 1);
        assert_eq!(msg.data, Some("test message".to_string()));

        // Check that sequence was recorded
        let received_seqs = receiver.get_received_seqs().await;
        assert!(received_seqs.contains(&1));
    }
}