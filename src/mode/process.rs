use crate::{
    net::{interface, namespace, nft, tc, veth},
    session::Session,
    spec::FaultSpec,
};
use anyhow::{Context, Result};
use std::{
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use tracing::info;

const HOST_ADDRESS: &str = "10.200.0.1/30";
const NAMESPACE_IP: &str = "10.200.0.2";
const NAMESPACE_ADDRESS: &str = "10.200.0.2/30";
const GATEWAY: &str = "10.200.0.1";
const NAT_CHAIN: &str = "postrouting";

pub fn run(spec: FaultSpec) -> Result<i32> {
    let target = &spec.target[0];
    let mut session = Session::default();
    let id = resource_id();
    let namespace_name = format!("{}{}", namespace::PREFIX, id);
    let host_name = format!("{}{}", veth::HOST_PREFIX, id);
    let ns_name = format!("{}{}", veth::NS_PREFIX, id);

    namespace::create(&namespace_name).context("creating process network namespace")?;
    session.namespace = Some(namespace_name.clone());

    let pair = veth::create(&host_name, &ns_name, &namespace_name)
        .context("creating process veth pair")?;
    session.veth = Some(pair);

    interface::address(&namespace_name, &ns_name, NAMESPACE_ADDRESS)
        .context("assigning namespace address")?;
    crate::net::run("ip", &["addr", "add", HOST_ADDRESS, "dev", &host_name])
        .context("assigning host veth address")?;
    interface::up(None, &host_name).context("bringing host veth up")?;
    interface::up(Some(&namespace_name), &ns_name).context("bringing namespace veth up")?;
    namespace::enable_loopback(&namespace_name).context("bringing namespace loopback up")?;
    interface::route(&namespace_name, GATEWAY).context("adding namespace default route")?;

    let previous = std::fs::read_to_string("/proc/sys/net/ipv4/ip_forward")?;
    std::fs::write("/proc/sys/net/ipv4/ip_forward", "1")?;
    session.forwarding_previous = Some(previous);
    let nat_table = format!("flt-nat-{id}");
    session.nat = Some(nft::NatResources {
        family: "ip".into(),
        table: nat_table.clone(),
        output_chain: NAT_CHAIN.into(),
        input_chain: String::new(),
    });
    nft::masquerade(&nat_table, NAT_CHAIN).context("creating process NAT")?;
    nft::allow_forward(&nat_table, &host_name).context("allowing process network forwarding")?;

    enable_local_port_forwarding(&mut session, &host_name, !spec.publish.is_empty())?;

    nft::publish_ports(&nat_table, NAMESPACE_IP, &spec.publish)
        .context("publishing process ports")?;
    tc::apply_netem(&host_name, &spec).context("applying host network faults")?;
    session.tc_interfaces.push(host_name.clone());
    tc::apply_netem_in_namespace(&namespace_name, &ns_name, &spec)
        .context("applying namespace network faults")?;

    let mut child_args = vec!["netns", "exec", &namespace_name, target];
    child_args.extend(spec.target.iter().skip(1).map(String::as_str));
    let mut child = Command::new("ip")
        .args(child_args)
        .spawn()
        .context("launching process in network namespace")?;
    let endpoints = spec
        .publish
        .iter()
        .map(|port| format!("http://{NAMESPACE_IP}:{}", port.container))
        .collect::<Vec<_>>();
    let address = if endpoints.is_empty() {
        format!("http://{NAMESPACE_IP}")
    } else {
        endpoints.join(",")
    };

    info!(
        target,
        namespace = %namespace_name,
        address = %address,
        "process fault session started"
    );
    let interrupted = Arc::new(AtomicBool::new(false));
    let signal = interrupted.clone();
    ctrlc::set_handler(move || {
        signal.store(true, Ordering::SeqCst);
    })
    .context("installing Ctrl+C handler")?;
    let status = loop {
        if interrupted.load(Ordering::SeqCst) {
            child.kill().ok();
        }
        if let Some(status) = child.try_wait()? {
            break status.code().unwrap_or(1);
        }
        thread::sleep(Duration::from_millis(50));
    };

    session
        .cleanup()
        .context("cleaning up process networking")?;
    info!(target, status, "process fault session stopped");
    Ok(status)
}

fn resource_id() -> String {
    format!("{:05}", std::process::id() % 100_000)
}

fn enable_local_port_forwarding(
    session: &mut Session,
    interface: &str,
    enabled: bool,
) -> Result<()> {
    if !enabled {
        return Ok(());
    }

    for path in [
        "/proc/sys/net/ipv4/conf/all/route_localnet".to_owned(),
        format!("/proc/sys/net/ipv4/conf/{interface}/route_localnet"),
    ] {
        let previous = std::fs::read_to_string(&path).with_context(|| format!("reading {path}"))?;
        std::fs::write(&path, "1").with_context(|| format!("enabling {path}"))?;
        session.route_localnet_previous.push((path, previous));
    }

    Ok(())
}
