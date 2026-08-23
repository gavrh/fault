use super::run;
use anyhow::{Result, anyhow};

pub fn default_route() -> Result<String> {
    let output = run("ip", &["-4", "route", "show", "default"])?;
    let words: Vec<_> = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    words
        .windows(2)
        .find(|w| w[0] == "dev")
        .map(|w| w[1].clone())
        .ok_or_else(|| anyhow!("could not determine the default network interface"))
}

pub fn up(namespace: Option<&str>, interface: &str) -> Result<()> {
    let mut args = vec!["link", "set", interface, "up"];
    if let Some(ns) = namespace {
        args = vec!["-n", ns, "link", "set", interface, "up"];
    }
    run("ip", &args)?;
    Ok(())
}

pub fn address(namespace: &str, interface: &str, value: &str) -> Result<()> {
    run(
        "ip",
        &["-n", namespace, "addr", "add", value, "dev", interface],
    )?;
    Ok(())
}

pub fn route(namespace: &str, gateway: &str) -> Result<()> {
    run(
        "ip",
        &["-n", namespace, "route", "add", "default", "via", gateway],
    )?;
    Ok(())
}
