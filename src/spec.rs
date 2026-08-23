use crate::cli::Cli;
use anyhow::{Result, bail};

#[derive(Debug)]
pub struct FaultSpec {
    pub target: Vec<String>,
    pub publish: Vec<PortForward>,
    pub corrupt: f32,
    pub duplicate: f32,
    pub jitter: u64,
    pub latency: u64,
    pub loss: f32,
    pub rate: u64,
    pub reorder: f32,
}

#[derive(Debug)]
pub struct PortForward {
    pub host: u16,
    pub container: u16,
}

impl FaultSpec {
    pub fn new(args: Cli) -> Result<Self> {
        let spec = FaultSpec {
            target: args.target,
            publish: args
                .publish
                .iter()
                .map(|value| parse_port_forward(value))
                .collect::<Result<Vec<_>>>()?,
            corrupt: args.corrupt,
            duplicate: args.duplicate,
            jitter: args.jitter,
            latency: args.latency,
            loss: args.loss,
            rate: args.rate,
            reorder: args.reorder,
        };

        spec.validate()?;

        Ok(spec)
    }

    fn validate(&self) -> Result<()> {
        for (name, value) in [
            ("corrupt", self.corrupt),
            ("duplicate", self.duplicate),
            ("loss", self.loss),
            ("reorder", self.reorder),
        ] {
            if !value.is_finite() || !(0.0..=100.0).contains(&value) {
                bail!("{name} must be between 0 and 100")
            }
        }
        Ok(())
    }

    pub fn netem_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.latency > 0 || self.jitter > 0 {
            args.extend(["delay".into(), format!("{}ms", self.latency)]);
            if self.jitter > 0 {
                args.push(format!("{}ms", self.jitter));
            }
        }
        for (name, value) in [
            ("loss", self.loss),
            ("duplicate", self.duplicate),
            ("corrupt", self.corrupt),
            ("reorder", self.reorder),
        ] {
            if value > 0.0 {
                args.extend([name.into(), format!("{value}%")]);
            }
        }

        if self.rate > 0 {
            args.extend(["rate".into(), format!("{}mbit", self.rate)]);
        }
        args
    }
}

fn parse_port_forward(value: &str) -> Result<PortForward> {
    let (host, container) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("publish must use HOST:CONTAINER, got `{value}`"))?;

    Ok(PortForward {
        host: host
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid host port in `{value}`"))?,
        container: container
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid container port in `{value}`"))?,
    })
}
