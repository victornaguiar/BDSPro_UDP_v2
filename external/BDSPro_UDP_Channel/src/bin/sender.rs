use tokio::net::UdpSocket;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use udp_channal_demo::Message;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    socket.connect("127.0.0.1:8080").await?;
    println!("UDP Sender is running. Enter messages:");

    let socket = Arc::new(socket);
    let sent_messages: Arc<Mutex<HashMap<u64, String>>> = Arc::new(Mutex::new(HashMap::new()));
    
    // Kanal für eingehende RQST-Nachrichten
    let (tx, mut rx) = mpsc::channel(100);  // multiple sender single consumer
    let recv_socket = socket.clone();  // neue Referenz auf socket

    // Asynchrone parallele tokio-Task im Hintergrund
    // Task: Empfangen von RQST nachrichten
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            if let Ok((len, _)) = recv_socket.recv_from(&mut buf).await {
                if let Ok(message) = serde_json::from_slice::<Message>(&buf[..len]) {
                    if message.typ == "RQST" {
                        tx.send(message.seq).await.ok(); // in den mpsc kanal gesendet
                    }
                }
            }
        }
    });

    let send_socket = socket.clone();
    let resend_map = sent_messages.clone();

    // Task: Reaktion auf RQSTs
    tokio::spawn(async move {
        while let Some(missing_seq) = rx.recv().await {
            // Asynchronen Lock auf den Mutex bekommen
            let map_guard = resend_map.lock().await;

            if let Some(text) = map_guard.get(&missing_seq) {
                let msg = Message {
                    typ: "DATA".to_string(),
                    seq: missing_seq,
                    data: Some(text.clone()),
                };
                let encoded = serde_json::to_vec(&msg).unwrap();
                send_socket.send(&encoded).await.unwrap();
                println!("[RESEND] SEQ {}", missing_seq);
            }
            // MutexGuard wird hier freigegeben, wenn es aus dem Scope fällt
        }
    });

    // Haupt-Loop: Eingabe von stdin
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    let mut seq = 1;

    while let Ok(Some(line)) = lines.next_line().await {
        let msg = Message {
            typ: "DATA".to_string(),
            seq,
            data: Some(line.clone()),
        };
        let mut map = sent_messages.lock().await;
        map.insert(seq, line);

        let encoded = serde_json::to_vec(&msg)?;
        socket.send(&encoded).await?;
        seq += 1;
    }

    Ok(())
}
