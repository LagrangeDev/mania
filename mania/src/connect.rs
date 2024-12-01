use std::io::Result;
use std::net::SocketAddr;

use crate::ping::ping;

/// Find the optimum server to connect to.
pub async fn optimum_server(request_msf: bool, ipv6: bool) -> Result<SocketAddr> {
    let addrs = if request_msf {
        resolve_dns(ipv6).await?
    } else if ipv6 {
        vec!["msfwifiv6.3g.qq.com:14000".parse().unwrap()]
    } else {
        vec![
            "183.47.102.193:8080".parse().unwrap(),
            "14.22.9.84:8080".parse().unwrap(),
            "119.147.190.138:8080".parse().unwrap(),
        ]
    };

    let mut results = ping(addrs, ipv6).await?;
    results.sort_by_key(|(_, rtt)| *rtt);

    Ok(results[0].0)
}

/// Resolve the DNS to get the server address.
async fn resolve_dns(ipv6: bool) -> Result<Vec<SocketAddr>> {
    let host = if ipv6 {
        "msfwifiv6.3g.qq.com:8080"
    } else {
        "msfwifi.3g.qq.com:8080"
    };
    let host = tokio::net::lookup_host(host).await?;
    Ok(host.collect())
}
