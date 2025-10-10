use host::api::{HostServer, HostServerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = HostServerConfig::from_yaml("example/host_server/config.yml");

    let server = HostServer::from_config(config);

    let _ = server.serve().await;

    Ok(())
}
