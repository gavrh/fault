use super::run;
use crate::spec::PortForward;
use anyhow::{Context, Result};

#[derive(Clone)]
pub struct NftResources {
    pub family: String,
    pub table: String,
    pub output_chain: String,
    pub input_chain: String,
}

pub type NatResources = NftResources;

pub fn create(resources: &NftResources, addresses: &[String]) -> Result<()> {
    run(
        "nft",
        &["add", "table", &resources.family, &resources.table],
    )?;
    run(
        "nft",
        &[
            "add",
            "chain",
            &resources.family,
            &resources.table,
            &resources.output_chain,
        ],
    )?;
    run(
        "nft",
        &[
            "add",
            "chain",
            &resources.family,
            &resources.table,
            &resources.input_chain,
        ],
    )?;
    for address in addresses {
        let protocol = if address.contains(':') { "ip6" } else { "ip" };
        run(
            "nft",
            &[
                "add",
                "rule",
                &resources.family,
                &resources.table,
                &resources.output_chain,
                protocol,
                "daddr",
                address,
                "meta",
                "mark",
                "set",
                "1",
            ],
        )?;
        run(
            "nft",
            &[
                "add",
                "rule",
                &resources.family,
                &resources.table,
                &resources.input_chain,
                protocol,
                "saddr",
                address,
                "meta",
                "mark",
                "set",
                "1",
            ],
        )?;
    }
    Ok(())
}
pub fn remove(resources: &NftResources) -> Result<()> {
    run(
        "nft",
        &["delete", "table", &resources.family, &resources.table],
    )
    .map(|_| ())
}

pub fn remove_nat(resources: &NatResources) -> Result<()> {
    run(
        "nft",
        &["delete", "table", &resources.family, &resources.table],
    )
    .map(|_| ())
}
pub fn masquerade(table: &str, chain: &str) -> Result<()> {
    run("nft", &["add", "table", "ip", table])?;
    run(
        "nft",
        &[
            "add",
            "chain",
            "ip",
            table,
            chain,
            "{",
            "type",
            "nat",
            "hook",
            "postrouting",
            "priority",
            "100",
            ";",
            "}",
        ],
    )?;
    run(
        "nft",
        &[
            "add",
            "rule",
            "ip",
            table,
            chain,
            "ip",
            "saddr",
            "10.200.0.0/30",
            "masquerade",
        ],
    )
    .map(|_| ())
}

pub fn allow_forward(table: &str, interface: &str) -> Result<()> {
    run(
        "nft",
        &[
            "add", "chain", "ip", table, "forward", "{", "type", "filter", "hook", "forward",
            "priority", "0", ";", "}",
        ],
    )
    .with_context(|| format!("creating forwarding chain for {interface}"))?;
    run(
        "nft",
        &[
            "add", "rule", "ip", table, "forward", "iifname", interface, "accept",
        ],
    )
    .with_context(|| format!("allowing traffic from {interface}"))?;
    run(
        "nft",
        &[
            "add", "rule", "ip", table, "forward", "oifname", interface, "accept",
        ],
    )
    .with_context(|| format!("allowing traffic to {interface}"))?;
    Ok(())
}

pub fn publish_ports(table: &str, namespace_address: &str, ports: &[PortForward]) -> Result<()> {
    if ports.is_empty() {
        return Ok(());
    }

    for (chain, hook, priority) in [
        ("prerouting", "prerouting", "-100"),
        ("output", "output", "-100"),
    ] {
        run(
            "nft",
            &[
                "add", "chain", "ip", table, chain, "{", "type", "nat", "hook", hook, "priority",
                priority, ";", "}",
            ],
        )
        .with_context(|| format!("creating nftables {chain} chain"))?;
    }

    for port in ports {
        let destination = format!("{namespace_address}:{}", port.container);
        let host_port = port.host.to_string();

        run(
            "nft",
            &[
                "add",
                "rule",
                "ip",
                table,
                "prerouting",
                "tcp",
                "dport",
                &host_port,
                "dnat",
                "to",
                &destination,
            ],
        )
        .with_context(|| format!("publishing TCP port {}", port.host))?;
        run(
            "nft",
            &[
                "add",
                "rule",
                "ip",
                table,
                "output",
                "ip",
                "daddr",
                "127.0.0.1",
                "tcp",
                "dport",
                &host_port,
                "dnat",
                "to",
                &destination,
            ],
        )
        .with_context(|| format!("publishing localhost TCP port {}", port.host))?;
    }

    Ok(())
}
