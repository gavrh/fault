mod destination;
mod process;

use crate::spec::FaultSpec;
use anyhow::Result;

pub enum Mode {
    Process,
    Destination,
}

impl Mode {
    pub fn from_targets(targets: &[String]) -> Self {
        if targets
            .first()
            .is_some_and(|target| std::path::Path::new(target).is_file())
        {
            Self::Process
        } else {
            Self::Destination
        }
    }
}

pub fn run(mode: Mode, spec: FaultSpec) -> Result<i32> {
    match mode {
        Mode::Process => process::run(spec),
        Mode::Destination => destination::run(spec),
    }
}
