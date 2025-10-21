use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rm-reth", about = "rm-reth CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Host server operations
    Host(host::command::Cli),
    /// Node operations
    Node(node::command::Cli),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Host(cmd) => cmd.run().await?,
        Commands::Node(cmd) => cmd.run().await?,
    }

    Ok(())
}
