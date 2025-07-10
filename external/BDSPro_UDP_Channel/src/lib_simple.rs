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

pub type Result<T> = std::result::Result<T, UdpChannelError>;

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

/// Simple UDP sender for testing
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
        let sent_messages = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            socket,
            sent_messages,
        })
    }

    /// Send a message
    pub async fn send_message(&self, seq: u64, data: String) -> Result<()> {
        let msg = Message {
            typ: "DATA".to_string(),
            seq,
            data: Some(data.clone()),
        };

        let mut map = self.sent_messages.lock().await;
        map.insert(seq, data);
        drop(map);

        let encoded = serde_json::to_vec(&msg)?;
        self.socket.send(&encoded).await?;
        Ok(())
    }

    /// Get local address
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        self.socket.local_addr().map_err(UdpChannelError::from)
    }
}

/// Simple UDP receiver for testing
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

    /// Receive a message
    pub async fn receive_message(&self) -> Result<Option<Message>> {
        let mut buf = [0u8; 1024];
        let (len, addr) = self.socket.recv_from(&mut buf).await?;
        let msg_bytes = &buf[..len];

        match serde_json::from_slice::<Message>(msg_bytes) {
            Ok(message) => {
                if message.typ == "DATA" {
                    let mut received_seqs = self.received_seqs.lock().await;
                    received_seqs.insert(message.seq);
                    drop(received_seqs);

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

    /// Request retransmission
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

    /// Get received sequences
    pub async fn get_received_seqs(&self) -> HashSet<u64> {
        self.received_seqs.lock().await.clone()
    }

    /// Get local address
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        self.socket.local_addr().map_err(UdpChannelError::from)
    }
}
