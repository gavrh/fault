pub mod dns;
pub mod ifb;
pub mod interface;
pub mod namespace;
pub mod nft;
pub mod tc;
pub mod veth;

use anyhow::{Context, Result, anyhow};
use std::process::{Command, Output};
use tracing::debug;

pub fn run(program: &str, args: &[&str]) -> Result<Output> {
    debug!(program, ?args, "running network command");
    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("running {program}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "{} failed: {}",
            program,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output)
}

pub fn require_commands() -> Result<()> {
    for command in ["ip", "tc", "nft"] {
        let paths: Vec<_> = std::env::var_os("PATH")
            .into_iter()
            .flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
            .collect();
        let found = paths.iter().any(|p| p.join(command).is_file());
        if !found {
            return Err(anyhow!(
                "required command `{command}` was not found in PATH"
            ));
        }
    }
    let uid = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|line| line.starts_with("Uid:"))?
                .split_whitespace()
                .nth(1)?
                .parse::<u32>()
                .ok()
        });
    if uid != Some(0) {
        return Err(anyhow!(
            "Fault requires root privileges to configure Linux networking"
        ));
    }
    Ok(())
}
