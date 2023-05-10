use serde::Deserialize;

use crate::error::{Result, TimeClaimError};

#[derive(Debug, Clone, Deserialize)]
pub struct BtcBlock {
    pub hash: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct ChainApiBlock {
    data: BtcBlock,
}

impl BtcBlock {
    pub async fn new() -> Result<Self> {
        Self::new_from_hash("latest").await
    }

    pub async fn new_from_hash(hash: &str) -> Result<Self> {
        let res: ChainApiBlock =
            reqwasm::http::Request::get(&format!("https://chain.api.btc.com/v3/block/{}", hash))
                .send()
                .await
                .map_err(|_| TimeClaimError::ChainApiError)?
                .json()
                .await
                .map_err(|_| TimeClaimError::DeserializeError)?;
        Ok(res.data)
    }
}
