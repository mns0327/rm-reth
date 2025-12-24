use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use node::manager::NodeManager;
use storage::TableId;
use tower::timeout::TimeoutLayer;
use tower::{BoxError, Service, ServiceBuilder};

use crate::command::{Command, Response};
use crate::layers::logging::LoggingLayer;

pub struct DispatcherConfig {
    // pub concurrency_limit: usize,
    pub timeout: Duration,
}

// TODO: error handler
pub fn build_dispatcher(
    dispatcher: Dispatcher,
    cfg: &DispatcherConfig,
) -> impl Service<Command, Response = Response, Error = BoxError> + Clone {
    ServiceBuilder::new()
        .layer(LoggingLayer)
        // .layer(ConcurrencyLimitLayer::new(cfg.concurrency_limit))
        .layer(TimeoutLayer::new(cfg.timeout))
        .service(dispatcher)
}

#[derive(Clone)]
pub struct Dispatcher {
    node: Arc<NodeManager>,
}

impl Dispatcher {
    pub fn new(node: Arc<NodeManager>) -> Self {
        Self { node }
    }
}

impl Service<Command> for Dispatcher {
    type Response = Response;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, cmd: Command) -> Self::Future {
        let node = self.node.clone();

        Box::pin(async move { handle_command(cmd, &node) })
    }
}

pub fn handle_command(cmd: Command, node: &NodeManager) -> Result<Response, anyhow::Error> {
    match cmd {
        // transection
        Command::SubmitTx(tx) => {
            node.push_transaction(tx)?;
            Ok(Response::Ok)
        }
        // Command::ValidateTransaction(tx) => {
        //     Ok(Response::Ok)
        // },

        // block
        // Command::ProposeBlock() => {},
        // Command::ImportBlock(block) => {},
        // Command::ValidateBlock(block) => {},

        // status
        Command::GetBalance(address) => {
            let balance = node
                .storage()
                .get_ref(TableId::Balance)
                .to_balance()
                .get_or_default(&address)?;

            Ok(Response::GetBalance(balance))
        }
        Command::GetNonce(address) => {
            let nonce = node
                .storage()
                .get_ref(TableId::Nonce)
                .to_nonce()
                .get_or_default(&address)?;

            Ok(Response::GetNonce(nonce))
        }
        // Command::QueryStateRoot() => {},

        // node
        Command::MineBlock(extra_data) => {
            let tx_pool = node.process_execution_transaction()?;

            let block = node.create_block_with_processed_tx_pool(tx_pool);

            node.mine_with_block(block, extra_data)?;

            Ok(Response::Ok)
        } // Command::SyncPeer(PeerId),
    }
}
