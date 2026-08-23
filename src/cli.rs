use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    /// Target process (./server) or destinations (github.com ...)
    pub target: Vec<String>,

    /// Packet corruption
    #[arg(
        long,
        short = 'c',
        value_name = "PERCENT",
        default_value_t = 0.0,
        hide_default_value = true
    )]
    pub corrupt: f32,

    /// Packet duplication
    #[arg(
        long,
        short = 'd',
        value_name = "PERCENT",
        default_value_t = 0.0,
        hide_default_value = true
    )]
    pub duplicate: f32,

    /// Network latency jitter
    #[arg(
        long,
        short = 'j',
        value_name = "MS",
        default_value_t = 0,
        hide_default_value = true
    )]
    pub jitter: u64,

    /// Added network latency
    #[arg(
        long,
        short = 't',
        value_name = "MS",
        default_value_t = 0,
        hide_default_value = true
    )]
    pub latency: u64,

    /// Network packet loss
    #[arg(
        long,
        short = 'l',
        value_name = "PERCENT",
        default_value_t = 0.0,
        hide_default_value = true
    )]
    pub loss: f32,

    /// Network rate limit
    #[arg(
        long,
        short = 'r',
        value_name = "MBIT",
        default_value_t = 0,
        hide_default_value = true
    )]
    pub rate: u64,

    /// Packet reordering
    #[arg(
        long,
        short = 'o',
        value_name = "PERCENT",
        default_value_t = 0.0,
        hide_default_value = true
    )]
    pub reorder: f32
}
