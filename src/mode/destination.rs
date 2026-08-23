use crate::{
    net::{dns, ifb, interface, nft, tc},
    session::Session,
    spec::FaultSpec,
};
use anyhow::{Context, Result};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use tracing::info;

pub fn run(spec: FaultSpec) -> Result<i32> {
    let addresses = dns::resolve(&spec.target).context("resolving destination targets")?;
    let interface = interface::default_route().context("determining default network interface")?;
    let id = format!("{:05}", std::process::id() % 100_000);
    let mut session = Session::default();

    let resources = nft::NftResources {
        family: "inet".into(),
        table: format!("flt-{}", id),
        output_chain: "output".into(),
        input_chain: "input".into(),
    };
    session.nft = Some(resources.clone());
    nft::create(&resources, &addresses).context("creating destination nftables rules")?;

    session.tc_interfaces.push(interface.clone());
    tc::apply_filtered(&interface, &addresses, &spec, false)
        .context("applying outbound destination faults")?;
    let ifb_name = format!("{}{}", ifb::PREFIX, id);
    ifb::create(&ifb_name).context("creating destination IFB device")?;
    session.ifb = Some(ifb_name.clone());

    tc::redirect_ingress(&interface, &ifb_name)
        .context("redirecting inbound destination traffic")?;
    session.tc_interfaces.push(ifb_name.clone());
    tc::apply_filtered(&ifb_name, &addresses, &spec, true)
        .context("applying inbound destination faults")?;

    info!(interface, targets = ?addresses, "destination fault session started");
    let stopped = Arc::new(AtomicBool::new(false));
    let signal = stopped.clone();
    ctrlc::set_handler(move || {
        signal.store(true, Ordering::SeqCst);
    })
    .context("installing Ctrl+C handler")?;
    while !stopped.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    session
        .cleanup()
        .context("cleaning up destination networking")?;
    info!(interface, "destination fault session stopped");
    Ok(0)
}
