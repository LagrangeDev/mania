use std::io::Result;
use std::net::SocketAddr;
use std::sync::{Arc, Weak};

use byteorder::{BigEndian, ByteOrder};
use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct PacketSender {
    packets_tx: Sender<Vec<u8>>,
    close_token: DropToken,
}

impl PacketSender {
    pub async fn send(&self, packet: Vec<u8>) -> Result<()> {
        if self.close_token.is_dropped() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "socket closed",
            ));
        }
        self.packets_tx.send(packet).await.expect("channel closed");
        Ok(())
    }
}

pub struct PacketReceiver {
    packets_rx: Receiver<Bytes>,
    close_token: DropToken,
}

impl PacketReceiver {
    pub async fn recv(&mut self) -> Result<Bytes> {
        if self.close_token.is_dropped() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "socket closed",
            ));
        }
        Ok(self.packets_rx.recv().await.expect("channel closed"))
    }
}

const CHANNEL_SIZE: usize = 32;

pub async fn connect(addr: SocketAddr) -> Result<(PacketSender, PacketReceiver)> {
    let stream = tokio::net::TcpStream::connect(addr).await?;

    let (read, write) = stream.into_split();

    let close_sync = DropSync::new();
    let close_token = close_sync.token();

    let (packets_tx, packets_rx) = {
        let (send_tx, send_rx) = tokio::sync::mpsc::channel(CHANNEL_SIZE);
        let (recv_tx, recv_rx) = tokio::sync::mpsc::channel(CHANNEL_SIZE);
        tokio::spawn(async move {
            let err = tokio::select! {
                Err(e) = send_loop(write, send_rx) => e,
                Err(e) = recv_loop(read, recv_tx) => e,
                else => return tracing::error!("socket closed"),
            };
            tracing::error!("Socket error: {}", err);
            drop(close_sync);
        });
        (send_tx, recv_rx)
    };

    let sender = PacketSender {
        packets_tx,
        close_token: close_token.clone(),
    };

    let receiver = PacketReceiver {
        packets_rx,
        close_token,
    };

    Ok((sender, receiver))
}

#[tracing::instrument(skip(stream, channel))]
async fn send_loop(mut stream: OwnedWriteHalf, mut channel: Receiver<Vec<u8>>) -> Result<()> {
    while let Some(packet) = channel.recv().await {
        stream.write_all(&packet).await?;
    }
    Ok(())
}

#[tracing::instrument(skip(stream, channel))]
async fn recv_loop(mut stream: OwnedReadHalf, channel: Sender<Bytes>) -> Result<()> {
    let mut buffer = BytesMut::new();
    loop {
        let packet_length = stream.read_u32().await?;
        if packet_length > 1024 * 1024 * 64 {
            tracing::error!("Packet too large");
            continue;
        }

        buffer.resize(packet_length as usize, 0);
        BigEndian::write_u32(&mut buffer, packet_length);
        stream.read_exact(&mut buffer[4..]).await?;

        if channel.send(buffer.split().freeze()).await.is_err() {
            return Ok(()); // dropped
        }
    }
}

#[derive(Clone)]
struct DropSync(Arc<()>);

impl DropSync {
    pub fn new() -> Self {
        Self(Arc::new(()))
    }

    /// Returns a token that can be used to check if the DropSync has been dropped
    pub fn token(&self) -> DropToken {
        DropToken(Arc::downgrade(&self.0))
    }
}

#[derive(Clone)]
struct DropToken(Weak<()>);

impl DropToken {
    /// Returns true if the all associated DropSync instances have been dropped
    pub fn is_dropped(&self) -> bool {
        self.0.strong_count() == 0
    }
}
