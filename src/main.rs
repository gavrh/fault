mod cli;

use cli::Cli;
use clap::Parser;

fn main() {
    let args = Cli::parse();
    
    // testing
    print!("Targets: ");
    for t in args.target {
        print!("{} ", t);
    }
    print!("\n");

    println!("Corrupt: {}", args.corrupt);
    println!("Duplicate: {}", args.duplicate);
    println!("Jitter: {}", args.jitter);
    println!("Latency: {}", args.latency);
    println!("Loss: {}", args.loss);
    println!("Rate: {}", args.rate);
    println!("Reorder: {}", args.reorder);
}
