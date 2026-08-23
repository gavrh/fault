use anyhow::{Context, Result};
use std::{collections::BTreeSet, net::ToSocketAddrs};

pub fn resolve(targets: &[String]) -> Result<Vec<String>> {
    let mut addresses = BTreeSet::new();

    for target in targets {
        let host = host_from_target(target);
        let resolved = resolve_host(host, target)?;

        addresses.extend(resolved.map(|address| address.ip().to_string()));
    }

    if addresses.is_empty() {
        anyhow::bail!("targets did not resolve to any IP addresses");
    }

    Ok(addresses.into_iter().collect())
}

fn host_from_target(target: &str) -> &str {
    target
        .strip_prefix("https://")
        .or_else(|| target.strip_prefix("http://"))
        .unwrap_or(target)
        .split('/')
        .next()
        .unwrap_or(target)
}

fn resolve_host<'a>(
    host: &'a str,
    target: &str,
) -> Result<impl Iterator<Item = std::net::SocketAddr> + 'a> {
    (host, 0)
        .to_socket_addrs()
        .with_context(|| format!("resolving {target}"))
}
