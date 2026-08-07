use anyhow::Result;
use clap::{Parser, ValueEnum};
use destiny_protocol_analyzer::{ingest, open_database, CapturePacket};
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    io::ErrorKind,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Mode {
    Capture,
    Replay,
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
enum HandshakeMode {
    Capture,
    ServerFirst,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HandshakeState {
    Connected,
    WaitingServerGreeting,
    WaitingClientResponse,
    HandshakeComplete,
    Failed,
    Closed,
}

struct HandshakeResponder {
    mode: HandshakeMode,
    greeting: Vec<u8>,
}
impl HandshakeResponder {
    fn from_env() -> Result<Self> {
        let mode = match std::env::var("HANDSHAKE_MODE")
            .unwrap_or_else(|_| "capture".into())
            .as_str()
        {
            "capture" => HandshakeMode::Capture,
            "server-first" | "server_first" => HandshakeMode::ServerFirst,
            other => {
                anyhow::bail!("unsupported HANDSHAKE_MODE={other}; use capture or server-first")
            }
        };
        let greeting = std::env::var("HANDSHAKE_GREETING").unwrap_or_default();
        let greeting = if greeting.is_empty() {
            Vec::new()
        } else {
            hex::decode(greeting.trim_start_matches("0x"))?
        };
        Ok(Self { mode, greeting })
    }
    async fn begin(&self, stream: &mut TcpStream, session: &str) -> Result<HandshakeState> {
        if self.mode == HandshakeMode::ServerFirst {
            if self.greeting.is_empty() {
                anyhow::bail!("server-first mode requires HANDSHAKE_GREETING hex")
            }
            stream.write_all(&self.greeting).await?;
            log_packet(session, "server", "server", &self.greeting, "sent greeting");
            println!(
                "state session={session} state=WAITING_CLIENT_RESPONSE bytes_sent={}",
                self.greeting.len()
            );
            Ok(HandshakeState::WaitingClientResponse)
        } else {
            println!("state session={session} state=WAITING_SERVER_GREETING bytes_sent=0");
            Ok(HandshakeState::WaitingServerGreeting)
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1:39000")]
    listen: String,
    #[arg(short, long, default_value = "protocol.db")]
    database: PathBuf,
    #[arg(long, value_enum, default_value_t = Mode::Capture)]
    mode: Mode,
    #[arg(long, default_value_t = 2)]
    handshake_timeout: u64,
    #[arg(long, value_enum)]
    handshake_mode: Option<HandshakeMode>,
    #[arg(long)]
    handshake_greeting: Option<String>,
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn ascii_dump(payload: &[u8]) -> String {
    payload
        .iter()
        .map(|b| {
            if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            }
        })
        .collect()
}
fn log_packet(session: &str, transport: &str, peer: &str, payload: &[u8], note: &str) {
    println!("packet timestamp_ms={} session_id={} transport={} peer={} bytes={} hex={} ascii={} note={}", timestamp_ms(), session, transport, peer, payload.len(), hex::encode(payload), ascii_dump(payload), note);
}

fn replay_response(db: &Connection, payload: &[u8]) -> Result<Option<Vec<u8>>> {
    let response = db.query_row(
        "SELECT s.payload FROM messages request JOIN packet_samples r ON r.message_id=request.id
         JOIN messages response ON response.session_id=request.session_id AND response.direction='server_to_client'
         JOIN packet_samples s ON s.message_id=response.id
         WHERE request.direction='client_to_server' AND r.payload=?1 AND response.timestamp >= request.timestamp
         ORDER BY response.timestamp, response.id LIMIT 1",
        params![payload], |row| row.get(0)).optional()?;
    Ok(response)
}

async fn serve_connection(
    mut stream: TcpStream,
    peer: String,
    db: &Connection,
    mode: Mode,
    handshake_timeout: Duration,
    responder: &HandshakeResponder,
) -> Result<()> {
    let session = format!("tcp-{}-{}", timestamp_ms(), peer.replace(':', "_"));
    println!(
        "accepted timestamp_ms={} session_id={session} peer={peer}",
        timestamp_ms()
    );
    let _state = responder.begin(&mut stream, &session).await?;
    let mut received = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        println!("waiting_for_payload session={session} peer={peer}");
        match timeout(handshake_timeout, stream.read(&mut buffer)).await {
            Ok(Ok(0)) => {
                if received.is_empty() {
                    log_packet(&session, "tcp", &peer, &[], "client closed before payload");
                    let id = ingest(
                        db,
                        &CapturePacket {
                            session_id: peer.clone(),
                            direction: "client_to_server".into(),
                            timestamp_ms: timestamp_ms(),
                            payload: Vec::new(),
                            opcode: None,
                            confidence: "unknown".into(),
                            notes: "empty capture; client closed before sending bytes".into(),
                        },
                    )?;
                    println!("stored_sample_id={id} session={session} peer={peer} bytes=0 state=CLOSED reason=client_closed");
                }
                println!("client_closed/reset peer={peer} bytes={}", received.len());
                break;
            }
            Ok(Ok(n)) => {
                received.extend_from_slice(&buffer[..n]);
                log_packet(&session, "tcp", &peer, &buffer[..n], "client payload");
                println!("received_bytes={n} session={session} peer={peer} total={} state=HANDSHAKE_COMPLETE", received.len());
                let id = ingest(
                    db,
                    &CapturePacket {
                        session_id: peer.clone(),
                        direction: "client_to_server".into(),
                        timestamp_ms: timestamp_ms(),
                        payload: buffer[..n].to_vec(),
                        opcode: None,
                        confidence: "unknown".into(),
                        notes: "raw harness observation; opcode unmapped".into(),
                    },
                )?;
                println!("stored_sample_id={id} session={session} peer={peer}");
                if matches!(mode, Mode::Replay) {
                    if let Some(response) = replay_response(db, &buffer[..n])? {
                        stream.write_all(&response).await?;
                        log_packet(&session, "tcp", &peer, &response, "replay response");
                        println!(
                            "sent bytes session={session} peer={peer} count={}",
                            response.len()
                        );
                    }
                }
            }
            Ok(Err(error))
                if matches!(
                    error.kind(),
                    ErrorKind::ConnectionReset | ErrorKind::BrokenPipe
                ) =>
            {
                println!("client_closed/reset session={session} peer={peer} bytes={} state=CLOSED reason=reset", received.len());
                break;
            }
            Ok(Err(error)) => return Err(error.into()),
            Err(_) => {
                println!(
                    "timeout session={session} peer={peer} bytes={} state=FAILED reason=timeout",
                    received.len()
                );
                break;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut responder = HandshakeResponder::from_env()?;
    if let Some(mode) = args.handshake_mode {
        responder.mode = mode;
    }
    if let Some(greeting) = args.handshake_greeting {
        responder.greeting = hex::decode(greeting.trim_start_matches("0x"))?;
    }
    let db = open_database(&args.database)?;
    let listener = TcpListener::bind(&args.listen).await?;
    println!(
        "destiny-rpcs3-harness listening on {} mode={:?}",
        listener.local_addr()?,
        args.mode
    );
    loop {
        let (stream, peer) = listener.accept().await?;
        serve_connection(
            stream,
            peer.to_string(),
            &db,
            args.mode,
            Duration::from_secs(args.handshake_timeout),
            &responder,
        )
        .await?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{io::AsyncReadExt, net::TcpListener};

    #[tokio::test]
    async fn client_sends_bytes_and_server_records_them() {
        let db = open_database(":memory:").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let client = tokio::spawn(async move { TcpStream::connect(address).await.unwrap() });
        let (server, peer) = listener.accept().await.unwrap();
        let mut client = client.await.unwrap();
        client.write_all(&[0, 1, 2]).await.unwrap();
        client.shutdown().await.unwrap();
        serve_connection(
            server,
            peer.to_string(),
            &db,
            Mode::Capture,
            Duration::from_secs(1),
            &HandshakeResponder {
                mode: HandshakeMode::Capture,
                greeting: Vec::new(),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            db.query_row("SELECT payload FROM packet_samples", [], |r| r
                .get::<_, Vec<u8>>(0))
                .unwrap(),
            vec![0, 1, 2]
        );
    }

    #[tokio::test]
    async fn client_closes_without_bytes_records_empty_capture() {
        let db = open_database(":memory:").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let client = tokio::spawn(async move { TcpStream::connect(address).await.unwrap() });
        let (server, peer) = listener.accept().await.unwrap();
        let mut client = client.await.unwrap();
        client.shutdown().await.unwrap();
        serve_connection(
            server,
            peer.to_string(),
            &db,
            Mode::Capture,
            Duration::from_secs(1),
            &HandshakeResponder {
                mode: HandshakeMode::Capture,
                greeting: Vec::new(),
            },
        )
        .await
        .unwrap();
        let (count, size): (i64, i64) = db
            .query_row(
                "SELECT count(*), coalesce(max(length(payload)), -1) FROM packet_samples",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((count, size), (1, 0));
    }

    #[tokio::test]
    async fn replay_response_sends_bytes() {
        let db = open_database(":memory:").unwrap();
        ingest(
            &db,
            &CapturePacket {
                session_id: "fixture".into(),
                direction: "client_to_server".into(),
                timestamp_ms: 1,
                payload: vec![7],
                opcode: None,
                confidence: "unknown".into(),
                notes: "fixture".into(),
            },
        )
        .unwrap();
        ingest(
            &db,
            &CapturePacket {
                session_id: "fixture".into(),
                direction: "server_to_client".into(),
                timestamp_ms: 2,
                payload: vec![8, 9],
                opcode: None,
                confidence: "unknown".into(),
                notes: "fixture".into(),
            },
        )
        .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let client = tokio::spawn(async move {
            let mut s = TcpStream::connect(address).await.unwrap();
            s.write_all(&[7]).await.unwrap();
            let mut b = [0; 2];
            s.read_exact(&mut b).await.unwrap();
            b
        });
        let (server, peer) = listener.accept().await.unwrap();
        serve_connection(
            server,
            peer.to_string(),
            &db,
            Mode::Replay,
            Duration::from_secs(1),
            &HandshakeResponder {
                mode: HandshakeMode::Capture,
                greeting: Vec::new(),
            },
        )
        .await
        .unwrap();
        assert_eq!(client.await.unwrap(), [8, 9]);
    }
}
