use std::str::FromStr;
use tracing_subscriber::fmt::format::FmtSpan;

pub fn init_tracing() {
    #[cfg(debug_assertions)]
    let level = "debug";
    #[cfg(not(debug_assertions))]
    let level = "info";

    let env_level = std::env::var("RUST_LOG").unwrap_or_else(|_| level.to_string());

    if tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::from_str(&env_level).unwrap_or(tracing::Level::INFO))
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_names(true)
            .with_span_events(FmtSpan::NONE)
            .finish(),
    )
    .is_err()
    {
        tracing::info!("tracing skipped")
    }
}
