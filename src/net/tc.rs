use super::run;
use crate::spec::FaultSpec;
use anyhow::{Context, Result};

pub fn apply_netem(interface: &str, spec: &FaultSpec) -> Result<()> {
    let args = spec.netem_args();
    if args.is_empty() {
        return Ok(());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut command = vec!["qdisc", "replace", "dev", interface, "root", "netem"];
    command.extend(refs);
    run("tc", &command).with_context(|| format!("applying netem to {interface}"))?;
    Ok(())
}

pub fn apply_netem_in_namespace(namespace: &str, interface: &str, spec: &FaultSpec) -> Result<()> {
    let args = spec.netem_args();
    if args.is_empty() {
        return Ok(());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut command = vec![
        "netns", "exec", namespace, "tc", "qdisc", "replace", "dev", interface, "root", "netem",
    ];
    command.extend(refs);
    run("ip", &command).with_context(|| format!("applying netem to {interface} in {namespace}"))?;
    Ok(())
}

pub fn apply_filtered(
    interface: &str,
    addresses: &[String],
    spec: &FaultSpec,
    source: bool,
) -> Result<()> {
    let args = spec.netem_args();
    if args.is_empty() {
        return Ok(());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut qdisc = vec![
        "qdisc", "replace", "dev", interface, "root", "handle", "1:", "prio", "bands", "2",
        "priomap", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1",
    ];
    run("tc", &qdisc).context("creating destination traffic priority qdisc")?;
    qdisc = vec![
        "qdisc", "replace", "dev", interface, "parent", "1:1", "handle", "10:", "netem",
    ];
    qdisc.extend(refs.clone());
    run("tc", &qdisc).context("creating destination netem qdisc")?;
    for (index, address) in addresses.iter().enumerate() {
        let direction = if source { "src_ip" } else { "dst_ip" };
        let protocol = if address.contains(':') { "ipv6" } else { "ip" };
        let priority = (10 + index).to_string();
        let filter = vec![
            "filter",
            "add",
            "dev",
            interface,
            "protocol",
            protocol,
            "parent",
            "1:",
            "prio",
            priority.as_str(),
            "flower",
            direction,
            address,
            "flowid",
            "1:1",
        ];
        run("tc", &filter)
            .with_context(|| format!("adding {direction} filter for {address} on {interface}"))?;
    }
    Ok(())
}

pub fn clear(interface: &str) -> Result<()> {
    let _ = run("tc", &["qdisc", "del", "dev", interface, "root"]);
    let _ = run("tc", &["qdisc", "del", "dev", interface, "ingress"]);
    Ok(())
}

pub fn redirect_ingress(interface: &str, ifb: &str) -> Result<()> {
    run(
        "tc",
        &[
            "qdisc", "add", "dev", interface, "handle", "ffff:", "ingress",
        ],
    )
    .with_context(|| format!("creating ingress qdisc on {interface}"))?;
    for (index, protocol) in ["ip", "ipv6"].iter().enumerate() {
        let priority = (10 + index).to_string();
        run(
            "tc",
            &[
                "filter",
                "add",
                "dev",
                interface,
                "parent",
                "ffff:",
                "protocol",
                protocol,
                "prio",
                priority.as_str(),
                "u32",
                "match",
                "u32",
                "0",
                "0",
                "action",
                "mirred",
                "egress",
                "redirect",
                "dev",
                ifb,
            ],
        )
        .with_context(|| format!("redirecting {protocol} ingress traffic to {ifb}"))?;
    }
    Ok(())
}
