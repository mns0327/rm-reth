use types::{
    Address, block::block::Block, bytes::FixedBytes, hash::Hash, int::Uint256,
    tx::transaction::Transaction,
};

#[derive(Debug)]
pub enum Command {
    // transection
    SubmitTx(Transaction),
    // ValidateTransaction(Transaction),

    // block
    // ProposeBlock,
    // ImportBlock(Block),
    // ValidateBlock(Block),

    // status
    GetBalance(Address),
    GetNonce(Address),

    // node
    MineBlock(FixedBytes<32>),
    // SyncPeer(PeerId),
}

#[derive(Debug)]
pub enum Response {
    Ok,
    TxReceipt(TxReceipt),
    Block(Block),
    GetBalance(Uint256),
    GetNonce(u64),
}

#[derive(Debug)]
pub struct TxReceipt {
    pub tx_hash: Hash,
    pub success: bool,
}

pub trait CommandLog {
    fn name(&self) -> &'static str;

    fn summary(&self) -> String;
}

impl CommandLog for Command {
    fn name(&self) -> &'static str {
        match self {
            Command::MineBlock { .. } => "mine_block",
            Command::SubmitTx(_) => "submit_tx",
            Command::GetBalance(_) => "get_balance",
            Command::GetNonce(_) => "get_nonce",
        }
    }

    fn summary(&self) -> String {
        match self {
            Command::MineBlock { .. } => "mine new block".into(),
            // TODO: tx display with tx hash
            Command::SubmitTx(tx) => format!("tx={:?}", tx),
            Command::GetBalance(addr) => format!("addr={}", addr),
            Command::GetNonce(addr) => format!("addr={}", addr),
        }
    }
}
