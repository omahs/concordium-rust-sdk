use crate::{
    contract_client::*,
    types::{transactions, RejectReason},
    v2::IntoBlockIdentifier,
};
pub use concordium_base::{cis2_types::MetadataUrl, cis4_types::*};
use concordium_base::{
    constants::MAX_PARAMETER_LEN,
    contracts_common::{self, AccountAddress},
    hashes::TransactionHash,
    smart_contracts::{ExceedsParameterSize, OwnedParameter},
    web3id::{CredentialHolderId, Web3IdSigner, REVOKE_DOMAIN_STRING},
};

#[derive(thiserror::Error, Debug)]
/// An error that can occur when executing CIS4 queries.
pub enum Cis4QueryError {
    /// The smart contract receive name is invalid.
    #[error("Invalid receive name: {0}")]
    InvalidReceiveName(#[from] contracts_common::NewReceiveNameError),

    /// A general RPC error occured.
    #[error("RPC error: {0}")]
    RPCError(#[from] super::v2::QueryError),

    /// The data returned from q query could not be parsed.
    #[error("Failed parsing the response.")]
    ResponseParseError(#[from] contracts_common::ParseError),

    /// The node rejected the invocation.
    #[error("Rejected by the node: {0:?}.")]
    NodeRejected(crate::types::RejectReason),
}

impl From<RejectReason> for Cis4QueryError {
    fn from(value: RejectReason) -> Self { Self::NodeRejected(value) }
}

#[derive(thiserror::Error, Debug)]
/// An error that can occur when sending CIS4 update transactions.
pub enum Cis4TransactionError {
    /// The smart contract receive name is invalid.
    #[error("Invalid receive name: {0}")]
    InvalidReceiveName(#[from] contracts_common::NewReceiveNameError),

    /// The parameter is too large.
    #[error("Parameter is too large: {0}")]
    InvalidParams(#[from] ExceedsParameterSize),

    /// A general RPC error occured.
    #[error("RPC error: {0}")]
    RPCError(#[from] super::v2::RPCError),

    /// The node rejected the invocation.
    #[error("Rejected by the node: {0:?}.")]
    NodeRejected(crate::types::RejectReason),
}

/// Transaction metadata for CIS-4 update transactions.
pub type Cis4TransactionMetadata = ContractTransactionMetadata;

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum Cis4Type {}

pub type Cis4Contract = ContractClient<Cis4Type>;

impl Cis4Contract {
    /// Look up an entry in the registry by its id.
    pub async fn credential_entry(
        &mut self,
        cred_id: CredentialHolderId,
        bi: impl IntoBlockIdentifier,
    ) -> Result<CredentialEntry, Cis4QueryError> {
        let parameter =
            OwnedParameter::from_serial(&cred_id).expect("Credential ID is a valid parameter.");

        self.view_raw("credentialEntry", parameter, bi).await
    }

    /// Look up the status of a credential by its id.
    pub async fn credential_status(
        &mut self,
        cred_id: CredentialHolderId,
        bi: impl IntoBlockIdentifier,
    ) -> Result<CredentialStatus, Cis4QueryError> {
        let parameter =
            OwnedParameter::from_serial(&cred_id).expect("Credential ID is a valid parameter.");

        self.view_raw("credentialStatus", parameter, bi).await
    }

    /// Get the list of all the revocation keys together with their nonces.
    pub async fn revocation_keys(
        &mut self,
        bi: impl IntoBlockIdentifier,
    ) -> Result<Vec<RevocationKeyWithNonce>, Cis4QueryError> {
        let parameter = OwnedParameter::empty();

        self.view_raw("revocationKeys", parameter, bi).await
    }

    /// Look up the issuer's metadata URL.
    pub async fn issuer_metadata(
        &mut self,
        bi: impl IntoBlockIdentifier,
    ) -> Result<MetadataUrl, Cis4QueryError> {
        let parameter = OwnedParameter::empty();
        self.view_raw("issuerMetadata", parameter, bi).await
    }

    /// Look up the issuer's account address.
    pub async fn issuer_address(
        &mut self,
        bi: impl IntoBlockIdentifier,
    ) -> Result<AccountAddress, Cis4QueryError> {
        let parameter = OwnedParameter::empty();

        self.view_raw("issuer", parameter, bi).await
    }

    /// Register a new credential.
    pub async fn register_credential(
        &mut self,
        signer: &impl transactions::ExactSizeTransactionSigner,
        metadata: &Cis4TransactionMetadata,
        cred_info: &CredentialInfo,
        additional_data: &[u8],
    ) -> Result<TransactionHash, Cis4TransactionError> {
        use contracts_common::Serial;
        let mut payload = contracts_common::to_bytes(cred_info);
        let actual = payload.len() + additional_data.len() + 2;
        if payload.len() + additional_data.len() + 2 > MAX_PARAMETER_LEN {
            return Err(Cis4TransactionError::InvalidParams(ExceedsParameterSize {
                actual,
                max: MAX_PARAMETER_LEN,
            }));
        }
        (additional_data.len() as u16)
            .serial(&mut payload)
            .expect("We checked lengths above, so this must succeed.");
        payload.extend_from_slice(additional_data);
        let parameter = OwnedParameter::try_from(payload)?;
        self.update_raw(signer, metadata, "registerCredential", parameter)
            .await
    }

    /// Revoke a credential as an issuer.
    pub async fn revoke_credential_as_issuer(
        &mut self,
        signer: &impl transactions::ExactSizeTransactionSigner,
        metadata: &Cis4TransactionMetadata,
        cred_id: CredentialHolderId,
        reason: Option<Reason>,
    ) -> Result<TransactionHash, Cis4TransactionError> {
        let parameter = OwnedParameter::from_serial(&(cred_id, reason))?;

        self.update_raw(signer, metadata, "revokeCredentialIssuer", parameter)
            .await
    }

    /// Revoke a credential as an issuer.
    ///
    /// The extra nonce that must be provided is the owner's nonce inside the
    /// contract. The signature on this revocation message is set to expire at
    /// the same time as the transaction.
    pub async fn revoke_credential_as_holder(
        &mut self,
        signer: &impl transactions::ExactSizeTransactionSigner,
        metadata: &Cis4TransactionMetadata,
        web3signer: impl Web3IdSigner, // the holder
        nonce: u64,
        reason: Option<Reason>,
    ) -> Result<TransactionHash, Cis4TransactionError> {
        use contracts_common::Serial;
        let mut to_sign = REVOKE_DOMAIN_STRING.to_vec();
        let cred_id = web3signer.id();
        cred_id
            .serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        self.address
            .serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        nonce
            .serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        metadata
            .expiry
            .seconds
            .checked_mul(1000)
            .unwrap_or(u64::MAX)
            .serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        reason
            .serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        let sig = web3signer.sign(&to_sign);
        let mut parameter_vec = sig.to_bytes().to_vec();
        parameter_vec.extend_from_slice(&to_sign[REVOKE_DOMAIN_STRING.len()..]);
        let parameter = OwnedParameter::try_from(parameter_vec)?;

        self.update_raw(signer, metadata, "revokeCredentialHolder", parameter)
            .await
    }

    /// Revoke a credential as a revoker.
    ///
    /// The extra nonce that must be provided is the owner's nonce inside the
    /// contract. The signature on this revocation message is set to expire at
    /// the same time as the transaction.
    pub async fn revoke_credential_as_revoker(
        &mut self,
        signer: &impl transactions::ExactSizeTransactionSigner,
        metadata: &Cis4TransactionMetadata,
        web3signer: impl Web3IdSigner, // the revoker.
        nonce: u64,
        key: RevocationKey,
        cred_id: CredentialHolderId,
        reason: Option<&Reason>,
    ) -> Result<TransactionHash, Cis4TransactionError> {
        use contracts_common::Serial;
        let mut to_sign = REVOKE_DOMAIN_STRING.to_vec();
        cred_id
            .serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        self.address
            .serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        nonce
            .serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        metadata
            .expiry
            .seconds
            .checked_mul(1000)
            .unwrap_or(u64::MAX)
            .serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        key.serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        reason
            .serial(&mut to_sign)
            .expect("Serialization to vector does not fail.");
        let sig = web3signer.sign(&to_sign);
        let mut parameter_vec = sig.to_bytes().to_vec();
        parameter_vec.extend_from_slice(&to_sign[REVOKE_DOMAIN_STRING.len()..]);
        let parameter = OwnedParameter::try_from(parameter_vec)?;

        self.update_raw(signer, metadata, "revokeCredentialHolder", parameter)
            .await
    }
}
