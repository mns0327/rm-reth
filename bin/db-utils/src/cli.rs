use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "db-utils", about = "Export redb tables to JSON", version)]
pub struct Cli {
    /// Path to redb database file
    #[arg(long)]
    pub db: String,

    /// Table names to export
    #[arg(long, default_values = vec!["block", "nonce", "balance"])]
    pub tables: Vec<String>,

    /// Output JSON file path
    #[arg(long)]
    pub out: String,
}
