use std::io::Result;
use std::net::SocketAddr;
use std::time::Duration;

use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};

/// Ping a list of addresses and return the latency.
pub async fn ping(addrs: Vec<SocketAddr>, ipv6: bool) -> Result<Vec<(SocketAddr, Duration)>> {
    let icmp = Client::new(&if ipv6 {
        Config::builder().kind(ICMP::V6).build()
    } else {
        Config::default()
    })?;

    let mut pingers = Vec::with_capacity(addrs.len());
    for (i, addr) in addrs.iter().enumerate() {
        let pinger = icmp.pinger(addr.ip(), PingIdentifier(i as u16)).await;
        pingers.push(pinger);
    }

    let mut results = Vec::with_capacity(addrs.len());
    let mut join_set = tokio::task::JoinSet::new();
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
