use super::run;
use anyhow::Result;

pub const PREFIX: &str = "flt-ns-";

pub fn create(name: &str) -> Result<()> {
    run("ip", &["netns", "add", name]).map(|_| ())
}
pub fn remove(name: &str) -> Result<()> {
    run("ip", &["netns", "del", name]).map(|_| ())
}
pub fn enable_loopback(name: &str) -> Result<()> {
    run("ip", &["-n", name, "link", "set", "lo", "up"]).map(|_| ())
}
