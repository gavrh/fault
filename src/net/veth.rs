use super::run;
use anyhow::Result;

pub const HOST_PREFIX: &str = "flt-h-";
pub const NS_PREFIX: &str = "flt-v-";
pub struct VethPair {
    pub host: String,
}

pub fn create(host: &str, namespace: &str, netns: &str) -> Result<VethPair> {
    run(
        "ip",
        &[
            "link", "add", host, "type", "veth", "peer", "name", namespace,
        ],
    )?;
    if let Err(error) = run("ip", &["link", "set", namespace, "netns", netns]) {
        let _ = run("ip", &["link", "del", host]);
        return Err(error);
    }
    Ok(VethPair { host: host.into() })
}
pub fn remove(pair: &VethPair) -> Result<()> {
    run("ip", &["link", "del", &pair.host]).map(|_| ())
}
