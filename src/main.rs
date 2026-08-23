mod cli;
mod mode;
mod net;
mod session;
mod spec;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    let args = Cli::parse();
    init_logging(args.verbose);

    if let Err(error) = run(args) {
        error!(error = ?error, "Fault failed");
        std::process::exit(1);
    }

    Ok(())
}

fn init_logging(verbose: u8) {
    let default_level = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::fmt().with_env_filter(filter).init();
    info!(verbose, "Fault starting");
}

fn run(args: Cli) -> Result<()> {
    let spec = spec::FaultSpec::new(args)?;

    net::require_commands()?;

    let mode = mode::Mode::from_targets(&spec.target);
    validate_mode(&mode, &spec)?;

    let status = mode::run(mode, spec)?;

    if status != 0 {
        std::process::exit(status);
    }

    Ok(())
}

fn validate_mode(mode: &mode::Mode, spec: &spec::FaultSpec) -> Result<()> {
    if matches!(mode, mode::Mode::Destination) && !spec.publish.is_empty() {
        anyhow::bail!("--publish is only supported for process targets");
    }

    Ok(())
}
