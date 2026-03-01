use crate::error::HostApiError;
use crate::server::{HostServer, HostServerConfig};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "host", about = "Host server commands")]
pub struct Cli {
    #[command(subcommand)]
    command: HostCommands,
}

#[derive(Subcommand)]
enum HostCommands {
    /// Run the p2p host server
    Serve {
        /// config.yml path
        #[arg(long)]
        config: PathBuf,
    },
}

impl Cli {
    pub async fn run(self) -> Result<(), HostApiError> {
        match self.command {
            HostCommands::Serve { config } => {
                println!("P2P Host server loading...");

                let config = HostServerConfig::from_yaml(config);

                let server = HostServer::from_config(config);

                types::init::init_tracing();
                server.serve().await?;
            }
        }
        Ok(())
    }
}
