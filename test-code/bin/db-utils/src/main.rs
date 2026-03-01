mod cli;
mod error;
mod export;

use clap::Parser;
use cli::Cli;
use error::Result;
use std::fs;
use storage::StorageManager;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let storage = StorageManager::create_or_open(&cli.db)?;
    let json_str = export::export_tables(&storage, &cli.tables)?;

    fs::write(&cli.out, json_str)?;
    println!("Exported to {}", cli.out);

    Ok(())
}
