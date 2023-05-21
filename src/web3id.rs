pub use concordium_base::web3id::*;
use concordium_base::{
    base::CredentialRegistrationID,
    cis4_types::{CredentialHolderId, CredentialStatus},
    contracts_common::{AccountAddress, ContractAddress},
    id::{constants::ArCurve, types::IpIdentity},
    web3id::{self, CredentialMetadata, CredentialsInputs},
};
use futures::TryStreamExt;

use crate::{
    cis4::{Cis4Contract, Cis4QueryError},
    v2::{self, BlockIdentifier},
};

#[derive(thiserror::Error, Debug)]
pub enum CredentialLookupError {
    #[error("Credential network not supported.")]
    IncorrectNetwork,
    #[error("Credential issuer not as stated: {stated} != {actual}.")]
    InconsistentIssuer {
        stated: IpIdentity,
        actual: IpIdentity,
    },
    #[error("Credential owner not as stated: {stated} != {actual}.")]
    InconsistentOwner {
        stated: CredentialHolderId,
        actual: CredentialHolderId,
    },
    #[error("Unable to look up account: {0}")]
    QueryError(#[from] v2::QueryError),
    #[error("Unable to query CIS4 contract: {0}")]
    Cis4QueryError(#[from] Cis4QueryError),
    #[error("Credential {cred_id} no longer present on account: {account}")]
    CredentialNotPresent {
        cred_id: CredentialRegistrationID,
        account: AccountAddress,
    },
    #[error("Initial credential {cred_id} cannot be used.")]
    InitialCredential { cred_id: CredentialRegistrationID },
    #[error(
        "Cannot parse the commitment returned from contract: {contract} for credential {cred_id}."
    )]
    CommitmentParseError {
        contract: ContractAddress,
        cred_id:  web3id::CredentialId,
    },
    #[error("Unexpected response from the node: {0}")]
    InvalidResponse(String),
}

pub struct CredentialWithMetadata {
    pub status:      CredentialStatus,
    pub commitments: CredentialsInputs<ArCurve>,
}

pub async fn verify_credential_metadata(
    mut client: v2::Client,
    network: web3id::did::Network,
    metadata: &ProofMetadata,
) -> Result<CredentialWithMetadata, CredentialLookupError> {
    if metadata.network != network {
        return Err(CredentialLookupError::IncorrectNetwork);
    }
    // TODO
    match metadata.cred_metadata {
        CredentialMetadata::Identity { issuer, cred_id } => {
            let ai = client
                .get_account_info(&cred_id.into(), BlockIdentifier::LastFinal)
                .await?;
            let Some(cred) = ai.response.account_credentials.values().find(|cred| cred.value.cred_id() == cred_id.as_ref()) else {
                return Err(CredentialLookupError::CredentialNotPresent{ cred_id, account: ai.response.account_address });
            };
            if cred.value.issuer() != issuer {
                return Err(CredentialLookupError::InconsistentIssuer {
                    stated: issuer,
                    actual: cred.value.issuer(),
                });
            }
            match &cred.value {
                concordium_base::id::types::AccountCredentialWithoutProofs::Initial { .. } => {
                    return Err(CredentialLookupError::InitialCredential { cred_id })
                }
                concordium_base::id::types::AccountCredentialWithoutProofs::Normal {
                    cdv,
                    commitments,
                } => {
                    let now = chrono::Utc::now();
                    let valid_from = cdv.policy.created_at.lower().ok_or_else(|| {
                        CredentialLookupError::InvalidResponse(
                            "Credential creation date is not valid.".into(),
                        )
                    })?;
                    let valid_until = cdv.policy.valid_to.upper().ok_or_else(|| {
                        CredentialLookupError::InvalidResponse(
                            "Credential creation date is not valid.".into(),
                        )
                    })?;
                    let status = if valid_from > now {
                        CredentialStatus::NotActivated
                    } else if valid_until < now {
                        CredentialStatus::Expired
                    } else {
                        CredentialStatus::Active
                    };
                    let commitments = CredentialsInputs::Identity {
                        commitments: commitments.cmm_attributes.clone(),
                    };

                    Ok(CredentialWithMetadata {
                        status,
                        commitments,
                    })
                }
            }
        }
        CredentialMetadata::Web3Id {
            contract,
            owner,
            id,
        } => {
            let mut contract_client = Cis4Contract::new(client, contract).await?;
            let entry = contract_client.credential_entry(id).await?;
            if entry.credential_info.holder_id != owner {
                return Err(CredentialLookupError::InconsistentOwner {
                    stated: owner,
                    actual: entry.credential_info.holder_id,
                });
            }
            let commitment = concordium_base::common::from_bytes(&mut std::io::Cursor::new(
                &entry.credential_info.commitment,
            ))
            .map_err(|_| CredentialLookupError::CommitmentParseError {
                contract,
                cred_id: id,
            })?;

            let commitments = CredentialsInputs::Web3 { commitment };

            let status = contract_client.credential_status(id).await?;

            Ok(CredentialWithMetadata {
                status,
                commitments,
            })
        }
    }
}

/// Retrieve the public data of credentials validating any metadata that is
/// part of the credentials.
///
/// If any credentials from the presentation are from a network different than
/// the one supplied an error is returned.
pub async fn get_public_data(
    client: &mut v2::Client,
    network: web3id::did::Network,
    presentation: &web3id::Presentation<ArCurve, web3id::Web3IdAttribute>,
) -> Result<Vec<CredentialWithMetadata>, CredentialLookupError> {
    let stream = presentation
        .metadata()
        .map(|meta| {
            let mainnet_client = client.clone();
            async move { verify_credential_metadata(mainnet_client, network, &meta).await }
        })
        .collect::<futures::stream::FuturesOrdered<_>>();
    Ok(stream.try_collect().await?)
}
