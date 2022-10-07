use druid_garden_chia_types::blockchain::coin::Coin;
use druid_garden_chia_types::blockchain::pending_payment::PendingPayment;
use druid_garden_chia_types::blockchain::transaction_record::TransactionRecord;
use druid_garden_chia_types::blockchain::wallet_balance::WalletBalance;
use druid_garden_chia_types::blockchain::wallet_info::WalletInfo;
use druid_garden_chia_types::blockchain::wallet_sync::WalletSync;
use reqwest::Client;
use serde_json::{json, Map};

use crate::clients::common::*;
use crate::clients::responses::{
    LoginResp, SignedTransactionRecordResp, TransactionRecordResp, WalletBalanceResp,
    WalletInfoResp, WalletSyncResp,
};

pub struct WalletClient {
    client: Client,
    host: String,
    port: u32,
}
impl WalletClient {
    pub fn new(host: &str, port: u32, ssl_path: &str) -> Self {
        WalletClient {
            client: get_client(ssl_path).unwrap_or_default(),
            host: host.to_string(),
            port,
        }
    }
    pub async fn log_in(&self, wallet_fingerprint: u32) -> Result<u32, std::io::Error> {
        let mut request_body = Map::new();
        request_body.insert("wallet_fingerprint".to_string(), json!(wallet_fingerprint));
        Ok(post::<LoginResp>(
            &self.client,
            &get_url(self.host.as_str(), self.port, "log_in"),
            &request_body,
        )
        .await?
        .fingerprint)
    }
    pub async fn log_in_and_skip(&self, wallet_fingerprint: u32) -> Result<u32, std::io::Error> {
        let mut request_body = Map::new();
        request_body.insert("wallet_fingerprint".to_string(), json!(wallet_fingerprint));
        Ok(post::<LoginResp>(
            &self.client,
            &get_url(self.host.as_str(), self.port, "log_in_and_skip"),
            &request_body,
        )
        .await?
        .fingerprint)
    }
    pub async fn get_wallets(&self) -> Result<Vec<WalletInfo>, std::io::Error> {
        Ok(post::<WalletInfoResp>(
            &self.client,
            &get_url(self.host.as_str(), self.port, "get_wallets"),
            &Map::new(),
        )
        .await?
        .wallets)
    }
    pub async fn get_wallet_balance(
        &self,
        wallet_id: u32,
    ) -> Result<Vec<WalletBalance>, std::io::Error> {
        let mut request_body = Map::new();
        request_body.insert("wallet_id".to_string(), json!(wallet_id));
        Ok(post::<WalletBalanceResp>(
            &self.client,
            &get_url(self.host.as_str(), self.port, "get_wallet_balance"),
            &request_body,
        )
        .await?
        .wallets)
    }
    pub async fn get_sync_status(&self) -> Result<WalletSync, std::io::Error> {
        let resp = post::<WalletSyncResp>(
            &self.client,
            &get_url(self.host.as_str(), self.port, "get_sync_status"),
            &Map::new(),
        )
        .await?;
        Ok(WalletSync {
            genesis_initialized: resp.genesis_initialized,
            synced: resp.synced,
            syncing: resp.syncing,
        })
    }
    pub async fn send_transaction(
        &self,
        wallet_id: u32,
        amount: u64,
        address: String,
        fee: u64,
    ) -> Result<TransactionRecord, std::io::Error> {
        let mut request_body = Map::new();
        request_body.insert("wallet_id".to_string(), json!(wallet_id));
        request_body.insert("amount".to_string(), json!(amount));
        request_body.insert("address".to_string(), json!(address));
        request_body.insert("fee".to_string(), json!(fee));
        Ok(post::<TransactionRecordResp>(
            &self.client,
            &get_url(self.host.as_str(), self.port, "send_transaction"),
            &request_body,
        )
        .await?
        .transaction)
    }
    pub async fn send_transaction_multi(
        &self,
        wallet_id: u32,
        additions: Vec<PendingPayment>,
        fee: u64,
    ) -> Result<TransactionRecord, std::io::Error> {
        let mut request_body = Map::new();
        request_body.insert("wallet_id".to_string(), json!(wallet_id));
        request_body.insert("additions".to_string(), json!(additions));
        request_body.insert("fee".to_string(), json!(fee));
        Ok(post::<TransactionRecordResp>(
            &self.client,
            &get_url(self.host.as_str(), self.port, "send_transaction_multi"),
            &request_body,
        )
        .await?
        .transaction)
    }
    pub async fn get_transaction(
        &self,
        wallet_id: u32,
        transaction_id: String,
    ) -> Result<TransactionRecord, std::io::Error> {
        let mut request_body = Map::new();
        request_body.insert("wallet_id".to_string(), json!(wallet_id));
        request_body.insert("transaction_id".to_string(), json!(transaction_id));
        Ok(post::<TransactionRecordResp>(
            &self.client,
            &get_url(self.host.as_str(), self.port, "get_transaction"),
            &request_body,
        )
        .await?
        .transaction)
    }
    pub async fn create_signed_transaction(
        &self,
        wallet_id: u32,
        additions: Vec<Coin>,
        coins: Vec<Coin>,
        fee: u64,
    ) -> Result<TransactionRecord, std::io::Error> {
        let mut request_body = Map::new();
        request_body.insert("wallet_id".to_string(), json!(wallet_id));
        request_body.insert("additions".to_string(), json!(additions));
        request_body.insert("coins".to_string(), json!(coins));
        request_body.insert("fee".to_string(), json!(fee));
        Ok(post::<SignedTransactionRecordResp>(
            &self.client,
            &get_url(self.host.as_str(), self.port, "create_signed_transaction"),
            &request_body,
        )
        .await?
        .signed_tx)
    }
}
