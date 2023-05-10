use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

use crate::btc::BtcBlock;
use crate::error::{Result, TimeClaimError};

pub type VerifiedStatus = bool;
pub const VERIFIED: VerifiedStatus = true;
pub const UNVERIFIED: VerifiedStatus = false;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeClaim<const V: VerifiedStatus> {
    pub timestamp: i64,
    evidence: Evidence,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
enum Evidence {
    BtcBlockHash(String),
}

impl TimeClaim<VERIFIED> {
    pub async fn new() -> Result<Self> {
        let block = BtcBlock::new().await?;
        Ok(Self::new_from_btc_block(block))
    }

    fn new_from_btc_block(block: BtcBlock) -> Self {
        Self {
            timestamp: block.timestamp,
            evidence: Evidence::BtcBlockHash(block.hash),
        }
    }
}

impl TimeClaim<UNVERIFIED> {
    pub async fn validate(self) -> Result<TimeClaim<VERIFIED>> {
        match &self.evidence {
            Evidence::BtcBlockHash(block_hash) => {
                let block = BtcBlock::new_from_hash(block_hash).await?;
                if block.timestamp != self.timestamp {
                    return Err(TimeClaimError::InvalidClaim);
                }
                Ok(TimeClaim::new_from_btc_block(block))
            }
        }
    }
}

impl std::str::FromStr for TimeClaim<UNVERIFIED> {
    type Err = TimeClaimError;

    fn from_str(payload: &str) -> Result<Self> {
        let bytes = general_purpose::URL_SAFE
            .decode(payload.as_bytes())
            .map_err(|_| TimeClaimError::BadPayload)?;
        let string = std::str::from_utf8(&bytes).map_err(|_| TimeClaimError::BadPayload)?;
        serde_json::from_str(string).map_err(|_| TimeClaimError::BadPayload)
    }
}

impl TimeClaim<VERIFIED> {
    pub fn as_b64(&self) -> String {
        let claim_str =
            serde_json::to_string(self).expect("serialize to succeed on verified claim");
        general_purpose::URL_SAFE.encode(claim_str.as_bytes())
    }
}
