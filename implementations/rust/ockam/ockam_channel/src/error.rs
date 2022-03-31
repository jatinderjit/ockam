use ockam_core::{
    errcode::{ErrorCode, Kind, Origin},
    thiserror, Error2,
};

/// Types of errors that may occur constructing a secure channel.
#[derive(Clone, Debug, thiserror::Error)]
pub enum SecureChannelError {
    /// The key exchange process failed.
    #[error("the key exchange process failed.")]
    KeyExchange = 1,
    /// Internal state is invalid.
    #[error("internal state is invalid.")]
    InvalidInternalState,
    /// Expected nonce was invalid.
    #[error("expected nonce was invalid.")]
    InvalidNonce,
    /// Key exchange process did not complete.
    #[error("key exchange process did not complete.")]
    KeyExchangeNotComplete,
    /// Invalid response received from the Hub.
    #[error("invalid response received from the Hub.")]
    InvalidHubResponse,
    /// Invalid LocalInfo type
    #[error("invalid LocalInfo type")]
    InvalidLocalInfoType,
}
#[allow(clippy::from_over_into)]
impl Into<Error2> for SecureChannelError {
    fn into(self) -> Error2 {
        use SecureChannelError::*;
        let kind = match self {
            KeyExchange | KeyExchangeNotComplete => Kind::Protocol,
            InvalidInternalState | InvalidNonce | InvalidHubResponse | InvalidLocalInfoType => {
                Kind::Invalid
            }
        };

        Error2::new(ErrorCode::new(Origin::Channel, kind), self)
    }
}
