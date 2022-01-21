use serde::{Deserialize};
use std::collections::HashMap;
use chia_types::blockchain::*;


#[derive(Deserialize)]
pub struct AdditionsAndRemovalsResp {
    pub additions: Vec<CoinRecord>,
    pub removals: Vec<CoinRecord>,
    pub success: bool
}

#[derive(Deserialize)]
pub struct BlockchainStateResp {
    pub blockchain_state: BlockchainState,
    pub success: bool
}

#[derive(Deserialize)]
pub struct BlockRecordResp {
    pub block_record: BlockRecord,
    pub success: bool
}

#[derive(Deserialize)]
pub struct BlockRecordAryResp {
    pub block_records: Vec<BlockRecord>,
    pub success: bool
}

#[derive(Deserialize)]
pub struct CoinRecordResp {
    pub coin_record: CoinRecord,
    pub success: bool
}

#[derive(Deserialize)]
pub struct CoinRecordAryResp {
    pub coin_records: Vec<CoinRecord>,
    pub success: bool
}

#[derive(Deserialize)]
pub struct CoinSpendResp {
    pub coin_solution: CoinSpend,
    pub success: bool
}

#[derive(Deserialize)]
pub struct FullBlockResp {
    pub block: FullBlock,
    pub success: bool
}

#[derive(Deserialize)]
pub struct FullBlockAryResp {
    pub blocks: Vec<FullBlock>,
    pub success: bool
}

#[derive(Deserialize)]
pub struct InitialFreezePeriodResp {
    pub initial_freeze_end_timestamp: u64,
    pub success: bool
}

#[derive(Deserialize)]
pub struct LoginResp {
    pub fingerprint: u32,
    pub success: bool
}


#[derive(Deserialize)]
pub struct MempoolItemResp {
    pub mempool_item: MemPoolItem,
    pub success: bool
}

#[derive(Deserialize)]
pub struct MempoolItemsResp {
    pub mempool_items: HashMap<String, MemPoolItem>,
    pub success: bool
}
#[derive(Deserialize)]
pub struct MempoolTXResp {
    pub tx_ids: Vec<String>,
    pub success: bool
}

#[derive(Deserialize)]
pub struct NetworkInfoResp {
    pub network_name: String,
    pub network_prefix: String,
    pub success: bool
}

#[derive(Deserialize)]
pub struct NetSpaceResp {
    pub space: u64,
    pub success: bool
}

#[derive(Deserialize)]
pub struct SignagePointOrEOSResp {
    pub signage_point: SignagePoint,
    pub eos: SubSlotBundle,
    pub time_received: f64,
    pub reverted: bool,
    pub success: bool
}

#[derive(Deserialize)]
pub struct SignedTransactionRecordResp {
    pub signed_tx: TransactionRecord,
    pub success: bool
}

#[derive(Deserialize)]
pub struct TXResp {
    pub status: TXStatus,
    pub success: bool
}

#[derive(Deserialize)]
pub struct TransactionRecordResp {
    pub transaction: TransactionRecord,
    pub success: bool
}

#[derive(Deserialize)]
pub struct UnfinishedBlockAryResp {
    pub headers: Vec<UnfinishedBlock>,
    pub success: bool
}

#[derive(Deserialize)]
pub struct WalletBalanceResp {
    pub wallets: Vec<WalletBalance>,
    pub success: bool
}

#[derive(Deserialize)]
pub struct WalletsResp {
    pub wallets: Vec<WalletInfo>,
    pub success: bool
}

#[derive(Deserialize)]
pub struct WalletSyncResp {
    pub genesis_initialized: bool,
    pub synced: bool,
    pub syncing: bool,
    pub success: bool
}