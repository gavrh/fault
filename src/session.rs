use anyhow::Result;

use crate::net::{namespace, nft, veth};

#[derive(Default)]
pub struct Session {
    pub namespace: Option<String>,
    pub veth: Option<veth::VethPair>,
    pub ifb: Option<String>,
    pub nft: Option<nft::NftResources>,
    pub forwarding_previous: Option<String>,
    pub route_localnet_previous: Vec<(String, String)>,
    pub nat: Option<nft::NatResources>,
    pub tc_interfaces: Vec<String>,
}

impl Session {
    pub fn cleanup(&mut self) -> Result<()> {
        let mut first_error = None;
        for interface in self.tc_interfaces.drain(..) {
            if let Err(error) = crate::net::tc::clear(&interface) {
                first_error.get_or_insert(error);
            }
        }
        if let Some(resources) = self.nat.take()
            && let Err(error) = nft::remove_nat(&resources)
        {
            first_error.get_or_insert(error);
        }
        if let Some(resources) = self.nft.take()
            && let Err(error) = nft::remove(&resources)
        {
            first_error.get_or_insert(error);
        }
        if let Some(ifb) = self.ifb.take()
            && let Err(error) = crate::net::ifb::remove(&ifb)
        {
            first_error.get_or_insert(error);
        }
        if let Some(pair) = self.veth.take()
            && let Err(error) = veth::remove(&pair)
        {
            first_error.get_or_insert(error);
        }
        if let Some(name) = self.namespace.take()
            && let Err(error) = namespace::remove(&name)
        {
            first_error.get_or_insert(error);
        }
        if let Some(previous) = self.forwarding_previous.take()
            && let Err(error) = std::fs::write("/proc/sys/net/ipv4/ip_forward", previous.trim())
        {
            first_error.get_or_insert(error.into());
        }
        for (path, previous) in self.route_localnet_previous.drain(..) {
            if let Err(error) = std::fs::write(path, previous.trim()) {
                first_error.get_or_insert(error.into());
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
