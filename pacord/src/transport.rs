use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

pub const MAX_CLIENTS: usize = 8;
pub const PROTOCOL_VERSION: u16 = 2;
const MAX_MESSAGE_BYTES: u32 = 8 * 1024 * 1024;
const CHALLENGE_BYTES: usize = 32;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FramePacket {
    pub sequence: u64,
    pub captured_at_micros: u64,
    pub width: u32,
    pub height: u32,
    pub jpeg: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
enum ClientMessage {
    Hello {
        protocol_version: u16,
        nickname: String,
        proof: [u8; 32],
    },
    Stop,
}

#[derive(Debug, Serialize, Deserialize)]
enum ServerMessage {
    Challenge { nonce: [u8; 32] },
    Accepted { session_id: String },
    Rejected { reason: String },
    Frame(FramePacket),
    Stopped,
}

#[derive(Debug)]
pub enum TransportError {
    Io(std::io::Error),
    Codec(Box<bincode::ErrorKind>),
    Authentication,
    Rejected(String),
    Protocol(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "erro de rede: {err}"),
            Self::Codec(err) => write!(f, "erro de protocolo: {err}"),
            Self::Authentication => write!(f, "autenticação PACORD recusada"),
            Self::Rejected(reason) => write!(f, "sessão rejeitada: {reason}"),
            Self::Protocol(reason) => write!(f, "protocolo inválido: {reason}"),
        }
    }
}

impl Error for TransportError {}

impl From<std::io::Error> for TransportError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<Box<bincode::ErrorKind>> for TransportError {
    fn from(value: Box<bincode::ErrorKind>) -> Self {
        Self::Codec(value)
    }
}

fn proof(secret: &[u8], nonce: &[u8; CHALLENGE_BYTES], nickname: &str) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC aceita qualquer chave não vazia");
    mac.update(nonce);
    mac.update(nickname.as_bytes());
    mac.update(&PROTOCOL_VERSION.to_be_bytes());
    mac.finalize().into_bytes().into()
}

async fn write_message<W, T>(writer: &mut W, message: &T) -> Result<(), TransportError>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let body = bincode::serialize(message)?;
    if body.len() > MAX_MESSAGE_BYTES as usize {
        return Err(TransportError::Protocol("mensagem excede o limite".into()));
    }
    writer.write_u32(body.len() as u32).await?;
    writer.write_all(&body).await?;
    writer.flush().await?;
    Ok(())
}

async fn read_message<R, T>(reader: &mut R) -> Result<T, TransportError>
where
    R: AsyncRead + Unpin,
    T: for<'de> Deserialize<'de>,
{
    let len = reader.read_u32().await?;
    if len == 0 || len > MAX_MESSAGE_BYTES {
        return Err(TransportError::Protocol(
            "tamanho de mensagem inválido".into(),
        ));
    }
    let mut body = vec![0u8; len as usize];
    reader.read_exact(&mut body).await?;
    Ok(bincode::deserialize(&body)?)
}

#[derive(Clone)]
pub struct FrameServer {
    bind_addr: SocketAddr,
    secret: Arc<Vec<u8>>,
    clients: Arc<AtomicUsize>,
}

impl FrameServer {
    pub fn new(bind_addr: SocketAddr, secret: Vec<u8>) -> Result<Self, TransportError> {
        if secret.len() < 16 {
            return Err(TransportError::Protocol(
                "o segredo do PACORD deve ter pelo menos 16 bytes".into(),
            ));
        }
        Ok(Self {
            bind_addr,
            secret: Arc::new(secret),
            clients: Arc::new(AtomicUsize::new(0)),
        })
    }

    pub async fn run(&self, frames: broadcast::Sender<FramePacket>) -> Result<(), TransportError> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        self.run_on_listener(listener, frames).await
    }

    async fn run_on_listener(
        &self,
        listener: TcpListener,
        frames: broadcast::Sender<FramePacket>,
    ) -> Result<(), TransportError> {
        loop {
            let (stream, peer) = listener.accept().await?;
            if self
                .clients
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                    (count < MAX_CLIENTS).then_some(count + 1)
                })
                .is_err()
            {
                let _ = reject_connection(stream, "limite de oito clientes atingido").await;
                continue;
            }

            let secret = self.secret.clone();
            let clients = self.clients.clone();
            let frames = frames.clone();
            tokio::spawn(async move {
                let result = handle_client(stream, secret, frames).await;
                clients.fetch_sub(1, Ordering::AcqRel);
                if let Err(error) = result {
                    log::warn!("cliente PACORD {peer} desconectado: {error}");
                }
            });
        }
    }
}

async fn reject_connection(mut stream: TcpStream, reason: &str) -> Result<(), TransportError> {
    write_message(
        &mut stream,
        &ServerMessage::Rejected {
            reason: reason.into(),
        },
    )
    .await
}

async fn handle_client(
    mut stream: TcpStream,
    secret: Arc<Vec<u8>>,
    frames: broadcast::Sender<FramePacket>,
) -> Result<(), TransportError> {
    let mut nonce = [0u8; CHALLENGE_BYTES];
    getrandom::fill(&mut nonce).map_err(|e| TransportError::Protocol(e.to_string()))?;
    write_message(&mut stream, &ServerMessage::Challenge { nonce }).await?;

    let hello: ClientMessage = read_message(&mut stream).await?;
    let ClientMessage::Hello {
        protocol_version,
        nickname,
        proof: received_proof,
    } = hello
    else {
        return Err(TransportError::Protocol(
            "primeira mensagem não é Hello".into(),
        ));
    };

    if protocol_version != PROTOCOL_VERSION || nickname.trim().is_empty() || nickname.len() > 64 {
        let _ = reject_connection(stream, "identificação de cliente inválida").await;
        return Err(TransportError::Rejected("identificação inválida".into()));
    }
    let expected = proof(&secret, &nonce, &nickname);
    if received_proof != expected {
        let _ = reject_connection(stream, "prova de posse do segredo inválida").await;
        return Err(TransportError::Authentication);
    }

    write_message(
        &mut stream,
        &ServerMessage::Accepted {
            session_id: Uuid::new_v4().to_string(),
        },
    )
    .await?;

    let mut receiver = frames.subscribe();
    loop {
        tokio::select! {
            frame = receiver.recv() => {
                match frame {
                    Ok(frame) => write_message(&mut stream, &ServerMessage::Frame(frame)).await?,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => {
                        write_message(&mut stream, &ServerMessage::Stopped).await?;
                        return Ok(());
                    }
                }
            }
            message = read_message::<_, ClientMessage>(&mut stream) => {
                match message? {
                    ClientMessage::Stop => {
                        write_message(&mut stream, &ServerMessage::Stopped).await?;
                        return Ok(());
                    }
                    ClientMessage::Hello { .. } => {
                        return Err(TransportError::Protocol("Hello repetido".into()));
                    }
                }
            }
        }
    }
}

pub async fn connect_client(
    server: SocketAddr,
    nickname: String,
    secret: Vec<u8>,
) -> Result<mpsc::Receiver<Result<FramePacket, TransportError>>, TransportError> {
    if secret.len() < 16 {
        return Err(TransportError::Protocol(
            "o segredo do PACORD deve ter pelo menos 16 bytes".into(),
        ));
    }
    let mut stream = TcpStream::connect(server).await?;
    let challenge: ServerMessage = read_message(&mut stream).await?;
    let ServerMessage::Challenge { nonce } = challenge else {
        return Err(TransportError::Protocol(
            "servidor não enviou desafio".into(),
        ));
    };
    write_message(
        &mut stream,
        &ClientMessage::Hello {
            protocol_version: PROTOCOL_VERSION,
            proof: proof(&secret, &nonce, &nickname),
            nickname,
        },
    )
    .await?;
    match read_message::<_, ServerMessage>(&mut stream).await? {
        ServerMessage::Accepted { .. } => {}
        ServerMessage::Rejected { reason } => return Err(TransportError::Rejected(reason)),
        _ => {
            return Err(TransportError::Protocol(
                "resposta de aceitação inválida".into(),
            ))
        }
    }

    let (sender, receiver) = mpsc::channel(2);
    tokio::spawn(async move {
        loop {
            let message = read_message::<_, ServerMessage>(&mut stream).await;
            match message {
                Ok(ServerMessage::Frame(frame)) => {
                    if sender.send(Ok(frame)).await.is_err() {
                        break;
                    }
                }
                Ok(ServerMessage::Stopped) => break,
                Ok(ServerMessage::Rejected { reason }) => {
                    let _ = sender.send(Err(TransportError::Rejected(reason))).await;
                    break;
                }
                Ok(ServerMessage::Challenge { .. }) | Ok(ServerMessage::Accepted { .. }) => {
                    let _ = sender
                        .send(Err(TransportError::Protocol("mensagem inesperada".into())))
                        .await;
                    break;
                }
                Err(error) => {
                    let _ = sender.send(Err(error)).await;
                    break;
                }
            }
        }
    });
    Ok(receiver)
}

#[cfg(test)]
mod tests {
    use super::{proof, PROTOCOL_VERSION};

    #[test]
    fn proof_depends_on_nonce_and_nickname() {
        let secret = b"pacord-test-secret-1234";
        let nonce = [7u8; 32];
        let first = proof(secret, &nonce, "alice");
        assert_eq!(first, proof(secret, &nonce, "alice"));
        assert_ne!(first, proof(secret, &nonce, "bob"));
        assert_ne!(first, proof(secret, &[8u8; 32], "alice"));
        assert_eq!(PROTOCOL_VERSION, 2);
    }

    #[tokio::test]
    async fn authenticated_client_receives_frame() {
        let secret = b"pacord-test-secret-1234".to_vec();
        let bind_addr = "127.0.0.1:0".parse().unwrap();
        let server = super::FrameServer::new(bind_addr, secret.clone()).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (frames, _) = tokio::sync::broadcast::channel(2);
        let server_task = tokio::spawn({
            let server = server.clone();
            let frames = frames.clone();
            async move { server.run_on_listener(listener, frames).await }
        });

        let mut receiver = super::connect_client(address, "alice".into(), secret)
            .await
            .expect("cliente deveria autenticar");
        frames
            .send(super::FramePacket {
                sequence: 42,
                captured_at_micros: 100,
                width: 2,
                height: 1,
                jpeg: vec![1, 2, 3],
            })
            .unwrap();

        let received = tokio::time::timeout(std::time::Duration::from_secs(2), receiver.recv())
            .await
            .expect("frame não chegou a tempo")
            .expect("canal foi encerrado")
            .expect("cliente recebeu erro");
        assert_eq!(received.sequence, 42);
        server_task.abort();
    }
}
