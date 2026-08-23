use super::run;
use anyhow::Result;

pub const PREFIX: &str = "flt-ifb-";
pub fn create(name: &str) -> Result<()> {
    run("ip", &["link", "add", name, "type", "ifb"])?;
    run("ip", &["link", "set", name, "up"])?;
    Ok(())
}
pub fn remove(name: &str) -> Result<()> {
    run("ip", &["link", "del", name]).map(|_| ())
}
