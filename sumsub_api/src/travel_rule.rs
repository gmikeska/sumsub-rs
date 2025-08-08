// src/travel_rule.rs

//! This module will contain the data structures for Travel Rule compliance.

use serde::{Deserialize, Serialize};

/// Represents the request to initiate the SDK for a Travel Rule transaction.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitiateSdkRequest {
    pub user_id: String,
    pub txn_info: TxnInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_in_secs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level_name: Option<String>,
}

/// Represents the transaction info for a Travel Rule transaction.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TxnInfo {
    pub applicant_wallet_address: String,
    pub counterparty_wallet_address: String,
    pub amount: f64,
    pub currency_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto_chain: Option<String>,
    pub direction: String,
}

/// Represents the response from initiating the SDK for a Travel Rule transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitiateSdkResponse {
    pub token: String,
    pub link: String,
    pub ttl_in_secs: u32,
}

/// Represents the request to patch a transaction with a chain transaction ID.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PatchTransactionRequest {
    pub payment_txn_id: String,
}

/// Represents the ownership status of a transaction.
#[derive(Debug)]
pub enum OwnershipStatus {
    Confirmed,
    Unconfirmed,
}

impl ToString for OwnershipStatus {
    fn to_string(&self) -> String {
        match self {
            OwnershipStatus::Confirmed => "confirmed".to_string(),
            OwnershipStatus::Unconfirmed => "unconfirmed".to_string(),
        }
    }
}

/// Represents a request to confirm wallet ownership.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum ConfirmWalletOwnershipRequest {
    ApplicantId {
        applicant_id: String,
    },
    ApplicantParticipant {
        applicant_participant: ApplicantParticipant,
    },
}

/// Represents an applicant participant in a wallet ownership confirmation.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantParticipant {
    pub full_name: String,
    pub external_user_id: String,
    #[serde(rename = "type")]
    pub participant_type: String,
}

/// Represents a request to import wallet addresses.
pub type ImportWalletAddressesRequest = Vec<WalletAddress>;

/// Represents a wallet address to be imported.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct WalletAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address_hash: Option<String>,
    pub asset: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
}

/// Represents the response from importing wallet addresses.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportWalletAddressesResponse {
    pub imported_count: u32,
    pub failed_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ImportError>>,
}

/// Represents an error that occurred during wallet address import.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportError {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
}
