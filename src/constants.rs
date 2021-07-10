use crate::types::Nonce;

/// Maximum size of a transaction payload.
pub const MAX_PAYLOAD_SIZE: u32 = 100 * 1024;

/// Minimum valid transaction nonce. Nonces must be strictly sequential starting
/// with [MIN_NONCE].
pub const MIN_NONCE: Nonce = Nonce { nonce: 1 };

/// Size of the sha256 digest in bytes.
pub const SHA256: usize = 32;

/// Maximum allowed size of data to register via the register data transaction.
pub const MAX_REGISTERED_DATA_SIZE: usize = 256;

/// Maximum allowed length of a smart contract parameter.
/// This must be kept in sync with maxParameterLen.
pub const MAX_PARAMETER_LEN: usize = 1024;

/// Maximum allowed size of the Wasm module to deploy on the chain.
pub const MAX_WASM_MODULE_SIZE: u32 = 65536;