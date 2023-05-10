use thiserror::Error;

pub type Result<T> = std::result::Result<T, TimeClaimError>;

#[derive(Error, Debug, Clone)]
pub enum TimeClaimError {
    #[error("error accessing chain.api.btc.com")]
    ChainApiError,
    #[error("deserialize error")]
    DeserializeError,
    #[error("bad validation payload")]
    BadPayload,
    #[error("invalid claim, timestamp mismatch")]
    InvalidClaim,
    #[error("qr production error")]
    Qr,
}
