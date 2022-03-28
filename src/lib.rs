use actix_web::client::{Client, ClientResponse, Connector};
use actix_web::dev::Decompress;
use actix_web::dev::Payload;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use druid_garden_chia_types::blockchain::block_record::BlockRecord;
use druid_garden_chia_types::blockchain::blockchain_state::BlockchainState;
use druid_garden_chia_types::blockchain::coin::Coin;
use druid_garden_chia_types::blockchain::coin_record::CoinRecord;
use druid_garden_chia_types::blockchain::coin_spend::CoinSpend;
use druid_garden_chia_types::blockchain::full_block::FullBlock;
use druid_garden_chia_types::blockchain::mem_pool_item::MemPoolItem;
use druid_garden_chia_types::blockchain::network_info::NetworkInfo;
use druid_garden_chia_types::blockchain::pending_payment::PendingPayment;
use druid_garden_chia_types::blockchain::signage_point_or_eos::SignagePointOrEOS;
use druid_garden_chia_types::blockchain::sized_bytes::Bytes32;
use druid_garden_chia_types::blockchain::spend_bundle::SpendBundle;
use druid_garden_chia_types::blockchain::transaction_record::TransactionRecord;
use druid_garden_chia_types::blockchain::tx_status::TXStatus;
use druid_garden_chia_types::blockchain::unfinished_block::UnfinishedBlock;
use druid_garden_chia_types::blockchain::wallet_balance::WalletBalance;
use druid_garden_chia_types::blockchain::wallet_info::WalletInfo;
use druid_garden_chia_types::blockchain::wallet_sync::WalletSync;
use openssl::ssl::{SslConnector, SslConnectorBuilder, SslFiletype, SslMethod, SslVerifyMode};
use serde_json::{json, Map};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

mod responses;

fn init_ssl(ssl_path: &str) -> SslConnector {
    let mut builder: SslConnectorBuilder = SslConnector::builder(SslMethod::tls()).unwrap();
    let ca_str = [ssl_path, "/daemon/private_daemon.crt"].concat();
    let ca_path = Path::new(ca_str.as_str());
    let key_str = [ssl_path, "/daemon/private_daemon.key"].concat();
    let key_path = Path::new(key_str.as_str());
    builder.set_certificate_chain_file(&ca_path).unwrap();
    builder
        .set_private_key_file(&key_path, SslFiletype::PEM)
        .unwrap();
    builder.set_verify(SslVerifyMode::NONE);
    builder.build()
}

fn get_url(host: &str, port: u32, request_uri: &str) -> String {
    format!(
        "https://{host}:{port}/{request_uri}",
        host = host,
        port = port,
        request_uri = request_uri
    )
}

fn get_client(connector: SslConnector) -> Client {
    Client::builder()
        .connector(Connector::new().ssl(connector).finish())
        .finish()
}

pub struct FullnodeClient {
    connector: SslConnector,
    host: String,
    port: u32,
}

impl FullnodeClient {
    pub fn new(host: &str, port: u32, ssl_path: &str) -> Self {
        FullnodeClient {
            connector: init_ssl(ssl_path),
            host: host.to_string(),
            port: port,
        }
    }

    pub async fn get_blockchain_state(&self) -> Result<BlockchainState, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_blockchain_state");
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body("{}")
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json_result: Result<responses::BlockchainStateResp, serde_json::Error> =
                    serde_json::from_str(body_str);
                if json_result.is_err() {
                    Err("Failed to Parse Json".into())
                } else {
                    let json: responses::BlockchainStateResp = json_result.ok().unwrap();
                    if json.success {
                        Ok(json.blockchain_state)
                    } else {
                        Err("Failed to Fetch Blockchain State".into())
                    }
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_block(&self, header_hash: &Bytes32) -> Result<FullBlock, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_block");
        let mut request_body = Map::new();
        request_body.insert("header_hash".to_string(), json!(header_hash));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::FullBlockResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.block)
                } else {
                    Err("Failed to Fetch FullBlock".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_blocks(
        &self,
        start: u32,
        end: u32,
        exclude_header_hash: bool,
    ) -> Result<Vec<FullBlock>, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_blocks");
        let mut request_body = Map::new();
        request_body.insert("start".to_string(), json!(start));
        request_body.insert("end".to_string(), json!(end));
        request_body.insert(
            "exclude_header_hash".to_string(),
            json!(if exclude_header_hash { "True" } else { "False" }),
        );
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::FullBlockAryResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.blocks)
                } else {
                    Err("Failed to Fetch FullBlock".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_all_blocks(
        &self,
        start: u32,
        end: u32,
    ) -> Result<Vec<FullBlock>, Box<dyn Error>> {
        self.get_blocks(start, end, true).await
    }
    pub async fn get_block_record_by_height(
        &self,
        height: u32,
    ) -> Result<BlockRecord, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_block_record_by_height");
        let mut request_body = Map::new();
        request_body.insert("height".to_string(), json!(height));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::BlockRecordResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.block_record)
                } else {
                    Err("Failed to Fetch FullBlock".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_block_record(
        &self,
        header_hash: &Bytes32,
    ) -> Result<BlockRecord, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_block_record");
        let mut request_body = Map::new();
        request_body.insert("header_hash".to_string(), json!(header_hash));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::BlockRecordResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.block_record)
                } else {
                    Err("Failed to Fetch FullBlock".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_block_records(
        &self,
        start: u32,
        end: u32,
    ) -> Result<Vec<BlockRecord>, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_block_records");
        let mut request_body = Map::new();
        request_body.insert("start".to_string(), json!(start));
        request_body.insert("end".to_string(), json!(end));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::BlockRecordAryResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.block_records)
                } else {
                    Err("Bad Failed to Fetch FullBlock".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_unfinished_block_headers(
        &self,
    ) -> Result<Vec<UnfinishedBlock>, Box<dyn Error>> {
        let url: String = get_url(
            self.host.as_str(),
            self.port,
            "get_unfinished_block_headers",
        );
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body("{}")
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::UnfinishedBlockAryResp =
                    serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.headers)
                } else {
                    Err("Bad Failed to Fetch FullBlock".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_network_space(
        &self,
        older_block_header_hash: &Bytes32,
        newer_block_header_hash: &Bytes32,
    ) -> Result<u64, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_network_space");
        let mut request_body = Map::new();
        request_body.insert(
            "older_block_header_hash".to_string(),
            json!(older_block_header_hash),
        );
        request_body.insert(
            "newer_block_header_hash".to_string(),
            json!(newer_block_header_hash),
        );
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::NetSpaceResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.space)
                } else {
                    Err("Bad Failed to Fetch FullBlock".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_network_space_by_height(
        &self,
        older_block_height: u32,
        newer_block_height: u32,
    ) -> Result<u64, Box<dyn Error>> {
        let older_block = self.get_block_record_by_height(older_block_height).await?;
        let newer_block = self.get_block_record_by_height(newer_block_height).await?;
        self.get_network_space(&older_block.header_hash, &newer_block.header_hash)
            .await
    }
    pub async fn get_additions_and_removals(
        &self,
        header_hash: &Bytes32,
    ) -> Result<(Vec<CoinRecord>, Vec<CoinRecord>), Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_additions_and_removals");
        let mut request_body = Map::new();
        request_body.insert("header_hash".to_string(), json!(header_hash));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::AdditionsAndRemovalsResp =
                    serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok((json.additions, json.removals))
                } else {
                    Err("Bad Failed to Fetch FullBlock".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_initial_freeze_period(&self) -> Result<u64, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_initial_freeze_period");
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body("{}")
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::InitialFreezePeriodResp =
                    serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.initial_freeze_end_timestamp)
                } else {
                    Err("Bad Failed to Fetch FullBlock".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_network_info(&self) -> Result<NetworkInfo, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_network_info");
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body("{}")
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::NetworkInfoResp = serde_json::from_str(body_str)?;
                if json.success {
                    Ok(NetworkInfo {
                        network_name: json.network_name,
                        network_prefix: json.network_prefix,
                    })
                } else {
                    Err("Bad Failed to Fetch FullBlock".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_recent_signage_point_or_eos(
        &self,
        sp_hash: Option<&Bytes32>,
        challenge_hash: Option<&Bytes32>,
    ) -> Result<SignagePointOrEOS, Box<dyn Error>> {
        let url: String = get_url(
            self.host.as_str(),
            self.port,
            "get_recent_signage_point_or_eos",
        );
        let mut request_body = Map::new();
        if sp_hash != None {
            request_body.insert("sp_hash".to_string(), json!(sp_hash));
        } else if challenge_hash != None {
            request_body.insert("challenge_hash".to_string(), json!(challenge_hash));
        }
        if sp_hash != None && challenge_hash != None {
            Err("InvalidArgument get_recent_signage_point_or_eos: One of sp_hash or challenge_hash must be None".into())
        } else {
            let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
                .post(url)
                .send_body(serde_json::to_string(&request_body)?)
                .await?;
            match resp.status() {
                StatusCode::OK => {
                    let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                    let body_str: &str = std::str::from_utf8(&body)?;
                    let json: responses::SignagePointOrEOSResp =
                        serde_json::from_str(body_str).unwrap();
                    if json.success {
                        Ok(SignagePointOrEOS {
                            signage_point: json.signage_point,
                            eos: json.eos,
                            time_received: json.time_received,
                            reverted: json.reverted,
                        })
                    } else {
                        Err("Bad Failed to Fetch FullBlock".into())
                    }
                }
                _ => Err("Bad Status Code".into()),
            }
        }
    }
    pub async fn get_coin_records_by_puzzle_hash(
        &self,
        puzzle_hash: &Bytes32,
        include_spent_coins: bool,
        start_height: u32,
        end_height: u32,
    ) -> Result<Vec<CoinRecord>, Box<dyn Error>> {
        let url: String = get_url(
            self.host.as_str(),
            self.port,
            "get_coin_records_by_puzzle_hash",
        );
        let mut request_body = Map::new();
        request_body.insert("puzzle_hash".to_string(), json!(puzzle_hash));
        request_body.insert(
            "include_spent_coins".to_string(),
            json!(include_spent_coins),
        );
        request_body.insert("start_height".to_string(), json!(start_height));
        request_body.insert("end_height".to_string(), json!(end_height));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::CoinRecordAryResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.coin_records)
                } else {
                    Err("Bad Failed to Fetch CoinRecords".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_coin_records_by_puzzle_hashes(
        &self,
        puzzle_hashes: Vec<&Bytes32>,
        include_spent_coins: bool,
        start_height: u32,
        end_height: u32,
    ) -> Result<Vec<CoinRecord>, Box<dyn Error>> {
        let url: String = get_url(
            self.host.as_str(),
            self.port,
            "get_coin_records_by_puzzle_hashes",
        );
        let mut request_body = Map::new();
        request_body.insert("puzzle_hashes".to_string(), json!(puzzle_hashes));
        request_body.insert(
            "include_spent_coins".to_string(),
            json!(include_spent_coins),
        );
        request_body.insert("start_height".to_string(), json!(start_height));
        request_body.insert("end_height".to_string(), json!(end_height));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::CoinRecordAryResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.coin_records)
                } else {
                    Err("Bad Failed to Fetch CoinRecords".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }

    pub async fn get_coin_record_by_name(
        &self,
        name: &Bytes32,
    ) -> Result<Option<CoinRecord>, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_coin_record_by_name");
        let mut request_body = Map::new();
        request_body.insert("name".to_string(), json!(name));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                println!("{}", body_str);
                let json: responses::CoinRecordResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.coin_record)
                } else {
                    //Err("Bad Failed to Fetch CoinRecord".into())
                    Ok(None)
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_coin_records_by_parent_ids(
        &self,
        parent_ids: Vec<&Bytes32>,
        include_spent_coins: bool,
        start_height: u32,
        end_height: u32,
    ) -> Result<Vec<CoinRecord>, Box<dyn Error>> {
        let url: String = get_url(
            self.host.as_str(),
            self.port,
            "get_coin_records_by_parent_ids",
        );
        let mut request_body = Map::new();
        request_body.insert("parent_ids".to_string(), json!(parent_ids));
        request_body.insert(
            "include_spent_coins".to_string(),
            json!(include_spent_coins),
        );
        request_body.insert("start_height".to_string(), json!(start_height));
        request_body.insert("end_height".to_string(), json!(end_height));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::CoinRecordAryResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.coin_records)
                } else {
                    Err("Bad Failed to Fetch CoinRecords".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn push_tx(&self, spend_bundle: &SpendBundle) -> Result<TXStatus, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "push_tx");
        let mut request_body = Map::new();
        request_body.insert("spend_bundle".to_string(), json!(spend_bundle));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::TXResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.status)
                } else {
                    Err("Bad Failed to Push Tx".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_puzzle_and_solution(
        &self,
        coin_id: &Bytes32,
        height: u32,
    ) -> Result<CoinSpend, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_puzzle_and_solution");
        let mut request_body = Map::new();
        request_body.insert("coin_id".to_string(), json!(coin_id));
        request_body.insert("height".to_string(), json!(height));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::CoinSpendResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.coin_solution)
                } else {
                    Err("Bad Failed to Fetch CoinRecords".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_coin_spend(
        &self,
        coin_record: &CoinRecord,
    ) -> Result<CoinSpend, Box<dyn Error>> {
        self.get_puzzle_and_solution(&coin_record.coin.name(), coin_record.spent_block_index)
            .await
    }
    pub async fn get_all_mempool_tx_ids(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_all_mempool_tx_ids");
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body("{}")
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::MempoolTXResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.tx_ids)
                } else {
                    Err("Bad Failed to Fetch CoinRecords".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_all_mempool_items(
        &self,
    ) -> Result<HashMap<String, MemPoolItem>, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_all_mempool_items");
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body("{}")
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::MempoolItemsResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.mempool_items)
                } else {
                    Err("Bad Failed to Fetch CoinRecords".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_mempool_item_by_tx_id(
        &self,
        tx_id: &str,
    ) -> Result<MemPoolItem, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_mempool_item_by_tx_id");
        let mut request_body = Map::new();
        request_body.insert("tx_id".to_string(), json!(tx_id));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::MempoolItemResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.mempool_item)
                } else {
                    Err("Bad Failed to Fetch CoinRecords".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
}

pub struct WalletClient {
    connector: SslConnector,
    host: String,
    port: u32,
}
impl WalletClient {
    pub fn new(host: &str, port: u32, ssl_path: &str) -> Self {
        WalletClient {
            connector: init_ssl(ssl_path),
            host: host.to_string(),
            port: port,
        }
    }
    pub async fn log_in(&self, wallet_fingerprint: u32) -> Result<u32, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "log_in");
        let mut request_body = Map::new();
        request_body.insert("wallet_fingerprint".to_string(), json!(wallet_fingerprint));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::LoginResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.fingerprint)
                } else {
                    Err("Failed to Log in to wallet".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn log_in_and_skip(&self, wallet_fingerprint: u32) -> Result<u32, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "log_in_and_skip");
        let mut request_body = Map::new();
        request_body.insert("wallet_fingerprint".to_string(), json!(wallet_fingerprint));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::LoginResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.fingerprint)
                } else {
                    Err("Failed to Log in to wallet".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_wallets(&self) -> Result<Vec<WalletInfo>, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_wallets");
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body("{}")
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::WalletsResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.wallets)
                } else {
                    Err("Failed to get wallets".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_wallet_balance(
        &self,
        wallet_id: u32,
    ) -> Result<Vec<WalletBalance>, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_wallet_balance");
        let mut request_body = Map::new();
        request_body.insert("wallet_id".to_string(), json!(wallet_id));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::WalletBalanceResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(json.wallets)
                } else {
                    Err("Failed to Log in to wallet".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_sync_status(&self) -> Result<WalletSync, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_sync_status");
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body("{}")
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::WalletSyncResp = serde_json::from_str(body_str).unwrap();
                if json.success {
                    Ok(WalletSync {
                        genesis_initialized: json.genesis_initialized,
                        synced: json.synced,
                        syncing: json.syncing,
                    })
                } else {
                    Err("Failed to get wallets".into())
                }
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn send_transaction(
        &self,
        wallet_id: u32,
        amount: u64,
        address: String,
        fee: u64,
    ) -> Result<TransactionRecord, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "send_transaction");
        let mut request_body = Map::new();
        request_body.insert("wallet_id".to_string(), json!(wallet_id));
        request_body.insert("amount".to_string(), json!(amount));
        request_body.insert("address".to_string(), json!(address));
        request_body.insert("fee".to_string(), json!(fee));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::TransactionRecordResp =
                    serde_json::from_str(body_str).unwrap();
                Ok(json.transaction)
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn send_transaction_multi(
        &self,
        wallet_id: u32,
        additions: Vec<PendingPayment>,
        fee: u64,
    ) -> Result<TransactionRecord, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "send_transaction_multi");
        let mut request_body = Map::new();
        request_body.insert("wallet_id".to_string(), json!(wallet_id));
        request_body.insert("additions".to_string(), json!(additions));
        request_body.insert("fee".to_string(), json!(fee));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::TransactionRecordResp =
                    serde_json::from_str(body_str).unwrap();
                Ok(json.transaction)
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn get_transaction(
        &self,
        wallet_id: u32,
        transaction_id: String,
    ) -> Result<TransactionRecord, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "get_transaction");
        let mut request_body = Map::new();
        request_body.insert("wallet_id".to_string(), json!(wallet_id));
        request_body.insert("transaction_id".to_string(), json!(transaction_id));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::TransactionRecordResp =
                    serde_json::from_str(body_str).unwrap();
                Ok(json.transaction)
            }
            _ => Err("Bad Status Code".into()),
        }
    }
    pub async fn create_signed_transaction(
        &self,
        wallet_id: u32,
        additions: Vec<Coin>,
        coins: Vec<Coin>,
        fee: u64,
    ) -> Result<TransactionRecord, Box<dyn Error>> {
        let url: String = get_url(self.host.as_str(), self.port, "create_signed_transaction");
        let mut request_body = Map::new();
        request_body.insert("wallet_id".to_string(), json!(wallet_id));
        request_body.insert("additions".to_string(), json!(additions));
        request_body.insert("coins".to_string(), json!(coins));
        request_body.insert("fee".to_string(), json!(fee));
        let mut resp: ClientResponse<Decompress<Payload>> = get_client(self.connector.clone())
            .post(url)
            .send_body(serde_json::to_string(&request_body)?)
            .await?;
        match resp.status() {
            StatusCode::OK => {
                let body: Bytes = resp.body().limit(1024 * 1024 * 50).await?;
                let body_str: &str = std::str::from_utf8(&body)?;
                let json: responses::SignedTransactionRecordResp =
                    serde_json::from_str(body_str).unwrap();
                Ok(json.signed_tx)
            }
            _ => Err("Bad Status Code".into()),
        }
    }
}
