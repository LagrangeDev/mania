use std::io::Result;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use surge_ping::{Client, Config, ICMP, PingIdentifier, PingSequence};
use tokio::net::TcpStream;
use tokio::task::JoinSet;

/// Ping a list of addresses and return the latency.
pub async fn ping(addrs: Vec<SocketAddr>, ipv6: bool) -> Result<Vec<(SocketAddr, Duration)>> {
    // 首先尝试ICMP ping
    let icmp = Client::new(&if ipv6 {
        Config::builder().kind(ICMP::V6).build()
    } else {
        Config::default()
    });

    match icmp {
        Ok(icmp) => ping_with_icmp(icmp, addrs).await,
        Err(e) => {
            tracing::debug!("ICMP ping failed: {}, fallback to TCP ping", e);
            ping_with_tcp(addrs).await
        }
    }
}

async fn ping_with_icmp(
    icmp: Client,
    addrs: Vec<SocketAddr>,
) -> Result<Vec<(SocketAddr, Duration)>> {
    let mut pingers = Vec::with_capacity(addrs.len());
    for (i, addr) in addrs.iter().enumerate() {
        let pinger = icmp.pinger(addr.ip(), PingIdentifier(i as u16)).await;
        pingers.push(pinger);
    }

    let mut results = Vec::with_capacity(addrs.len());
    let mut join_set = JoinSet::new();
    for (mut pinger, addr) in pingers.into_iter().zip(addrs) {
        join_set.spawn(async move {
            let latency = pinger
                .ping(PingSequence(0), &[])
                .await
                .map_or(Duration::MAX, |(_, d)| d);
            (addr, latency)
        });
    }
    while let Some(result) = join_set.join_next().await {
        results.push(result?);
    }

    Ok(results)
}

async fn ping_with_tcp(addrs: Vec<SocketAddr>) -> Result<Vec<(SocketAddr, Duration)>> {
    let mut results = Vec::with_capacity(addrs.len());
    let mut join_set = JoinSet::new();

    for addr in addrs {
        join_set.spawn(async move {
            let start = Instant::now();
            let timeout = Duration::from_secs(1);
            let latency = match tokio::time::timeout(timeout, TcpStream::connect(addr)).await {
                Ok(Ok(_)) => start.elapsed(),
                _ => Duration::MAX,
            };
            (addr, latency)
        });
    }

    while let Some(result) = join_set.join_next().await {
        results.push(result?);
    }

    Ok(results)
}
