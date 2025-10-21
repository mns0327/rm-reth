use host::api::{HostServer, HostServerConfig};
use node::api::{Node, NodeConfig};
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_handler = tokio::spawn(async {
        if let Err(e) = async {
            let config = HostServerConfig::from_yaml("example/host_server/config.yml");
            let server = HostServer::from_config(config);

            server.serve().await?;
            Ok::<(), anyhow::Error>(())
        }
        .await
        {
            eprintln!("Server task failed: {:?}", e);
        }
    });

    sleep(Duration::from_millis(500)).await;

    let _node1_handler = tokio::spawn(async {
        if let Err(e) = async {
            let config = NodeConfig::from_yaml("example/node1/config.yml");
            let mut node = Node::from_config(config);

            node.serve().await?;
            Ok::<(), anyhow::Error>(())
        }
        .await
        {
            eprintln!("Node task failed: {:?}", e);
        }
    });

    sleep(Duration::from_millis(500)).await;

    let _node2_handler = tokio::spawn(async {
        if let Err(e) = async {
            let config = NodeConfig::from_yaml("example/node2/config.yml");
            let mut node = Node::from_config(config);

            node.serve().await?;
            Ok::<(), anyhow::Error>(())
        }
        .await
        {
            eprintln!("Node task failed: {:?}", e);
        }
    });

    server_handler.await?;

    Ok(())
}
