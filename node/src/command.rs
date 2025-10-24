use crate::{
    error::NodeError,
    server::{Node, NodeConfig},
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "node", about = "node server commands")]
pub struct Cli {
    #[command(subcommand)]
    command: NodeCommands,
}

#[derive(Subcommand)]
enum NodeCommands {
    /// Run the Node
    Serve {
        /// config.yml path
        #[arg(long)]
        config: PathBuf,
    },
}

impl Cli {
    pub async fn run(self) -> Result<(), NodeError> {
        match self.command {
            NodeCommands::Serve { config } => {
                println!("Node loading...");

                let config = NodeConfig::from_yaml(config);

                let mut node = Node::from_config(config);

                common::init_tracing();
                node.serve().await?;
            }
        }
        Ok(())
    }
}
