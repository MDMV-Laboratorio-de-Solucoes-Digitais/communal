//! Command-line interface for the communal community detection framework.
//!
//! Provides subcommands for running community detection, comparing algorithms,
//! and converting between graph file formats.

use clap::{Parser, Subcommand};

/// Command-line interface for the communal community detection framework.
#[derive(Parser)]
#[command(name = "communal", about = "Community detection framework")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Run community detection on a graph file
    Run {
        /// Input graph file
        #[arg(short, long)]
        input: String,
        /// Algorithm to use
        #[arg(short, long, default_value = "leiden")]
        algorithm: String,
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
        /// Resolution parameter
        #[arg(short, long, default_value_t = 1.0)]
        gamma: f64,
        /// Random seed
        #[arg(short, long)]
        seed: Option<u64>,
    },
    /// Compare multiple algorithms
    Compare {
        /// Input graph file
        #[arg(short, long)]
        input: String,
        /// Algorithm names to compare
        #[arg(short, long)]
        algorithms: Vec<String>,
    },
    /// Convert between graph formats
    Convert {
        /// Input graph file
        #[arg(short, long)]
        input: String,
        /// Output graph file
        #[arg(short, long)]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            input,
            algorithm,
            output,
            gamma,
            seed,
        } => {
            println!("Running {algorithm} on {input}");
            let _ = (output, gamma, seed);
        }
        Commands::Compare { input, algorithms } => {
            println!("Comparing {algorithms:?} on {input}");
        }
        Commands::Convert { input, output } => {
            println!("Converting {input} to {output}");
        }
    }
}
