use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex};

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
}

pub type Result<T> = std::result::Result<T, UdpChannelError>;

/// UDP sender that handles message transmission and retransmission
pub struct UdpSender {
    socket: Arc<UdpSocket>,
    sent_messages: Arc<Mutex<HashMap<u64, String>>>,
}

impl UdpSender {
    /// Create a new UDP sender
    pub async fn new(bind_addr: &str, connect_addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        socket.connect(connect_addr).await?;
        let socket = Arc::new(socket);
        let sent_messages = Arc::new(Mutex::new(HashMap::<u64, String>::new()));

        // Start background task to handle retransmission requests
        let recv_socket = socket.clone();
        let resend_map = sent_messages.clone();
        let (tx, mut rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                if let Ok((len, _)) = recv_socket.recv_from(&mut buf).await {
                    if let Ok(message) = serde_json::from_slice::<Message>(&buf[..len]) {
                        if message.typ == "RQST" {
                            let _ = tx.send(message.seq).await;
                        }
                    }
                }
            }
        });

        let send_socket = socket.clone();
        tokio::spawn(async move {
            while let Some(missing_seq) = rx.recv().await {
                let map_guard = resend_map.lock().await;
                if let Some(text) = map_guard.get(&missing_seq) {
                    let msg = Message {
                        typ: "DATA".to_string(),
                        seq: missing_seq,
                        data: Some(text.clone()),
                    };
                    if let Ok(encoded) = serde_json::to_vec(&msg) {
                        let _ = send_socket.send(&encoded).await;
                    }
                }
            }
        });

        Ok(Self {
            socket,
            sent_messages,
        })
    }

    /// Send a message with the given sequence number
    pub async fn send_message(&self, seq: u64, data: String) -> Result<()> {
        let msg = Message {
            typ: "DATA".to_string(),
            seq,
            data: Some(data.clone()),
        };

        let mut map = self.sent_messages.lock().await;
        map.insert(seq, data);

        let encoded = serde_json::to_vec(&msg)?;
        self.socket.send(&encoded).await?;
        Ok(())
    }

    /// Get the local socket address
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        self.socket.local_addr().map_err(UdpChannelError::from)
    }
}

/// UDP receiver that handles message reception and acknowledgment
pub struct UdpReceiver {
    socket: Arc<UdpSocket>,
    received_seqs: Arc<Mutex<HashSet<u64>>>,
}

impl UdpReceiver {
    /// Create a new UDP receiver
    pub async fn new(bind_addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        let socket = Arc::new(socket);
        let received_seqs = Arc::new(Mutex::new(HashSet::new()));

        Ok(Self {
            socket,
            received_seqs,
        })
    }

    /// Receive and process messages
    pub async fn receive_message(&self) -> Result<Option<Message>> {
        let mut buf = [0u8; 1024];
        let (len, addr) = self.socket.recv_from(&mut buf).await?;
        let msg_bytes = &buf[..len];

        match serde_json::from_slice::<Message>(msg_bytes) {
            Ok(message) => {
                if message.typ == "DATA" {
                    // Record that we've seen this sequence number
                    let mut received_seqs = self.received_seqs.lock().await;
                    received_seqs.insert(message.seq);

                    // Send back an ACK
                    let ack = Message {
                        typ: "ACK".to_string(),
                        seq: message.seq,
                        data: None,
                    };
                    let ack_encoded = serde_json::to_vec(&ack)?;
                    self.socket.send_to(&ack_encoded, addr).await?;

                    Ok(Some(message))
                } else {
                    Ok(Some(message))
                }
            }
            Err(e) => Err(UdpChannelError::Serialization(e)),
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