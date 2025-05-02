use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// test command
    Test {
        /// string for echo
        test_string: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Test { test_string } => {
            println!("test_string is {}", test_string);
        }
    }
}
