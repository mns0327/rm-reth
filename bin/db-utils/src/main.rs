mod cli;
mod error;
mod export;

use cli::Cli;
use clap::Parser;
use error::Result;
use storage::StorageManager;
use std::fs;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let storage = StorageManager::create_or_open(&cli.db)?;
    let json_str = export::export_tables(&storage, &cli.tables)?;

    fs::write(&cli.out, json_str)?;
    println!("Exported to {}", cli.out);

    Ok(())
}
