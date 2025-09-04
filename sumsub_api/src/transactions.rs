// src/transactions.rs

//! This module will contain the data structures for transaction monitoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the request to submit a new transaction.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubmitTransactionRequest {
    pub txn_id: String,
    pub txn_date: String,
    #[serde(rename = "type")]
    pub txn_type: String,
    pub applicant: TransactionApplicant,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_platform_event_info: Option<UserPlatformEventInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<TransactionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterparty: Option<TransactionApplicant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<HashMap<String, String>>,
}

/// Represents the applicant or counterparty in a transaction.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionApplicant {
    #[serde(rename = "type")]
    pub applicant_type: String,
    pub external_user_id: String,
    pub full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_of_birth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dob: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<TransactionAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<Device>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub institution_info: Option<InstitutionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<PaymentMethod>,
}

/// Represents the address of a transaction participant.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionAddress {
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub town: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flat_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted_address: Option<String>,
}

/// Represents the device of a transaction participant.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coords: Option<Coords>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_info: Option<IpInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_age_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_lang: Option<String>,
}

/// Represents the coordinates of a device.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Coords {
    pub lat: f64,
    pub lon: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<String>,
}

/// Represents the IP information of a device.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct IpInfo {
    pub ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lon: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code3: Option<String>,
}

/// Represents the information about a user platform event.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserPlatformEventInfo {
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Represents the general information about a transaction.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfo {
    pub direction: String,
    pub amount: f64,
    pub currency_code: String,
    pub currency_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_txn_id: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto_params: Option<CryptoParams>,
}

/// Represents the crypto parameters of a transaction.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CryptoParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto_chain: Option<String>,
}

/// Represents the information about a financial institution.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<TransactionAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<String>,
}

/// Represents a payment method.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethod {
    #[serde(rename = "type")]
    pub payment_type: String,
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuing_country: Option<String>,
}

/// Represents the response from submitting a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubmitTransactionResponse {
    pub id: String,
    pub created_at: String,
    pub client_id: String,
    pub applicant_id: String,
    pub txn_id: String,
    #[serde(rename = "type")]
    pub txn_type: String,
    pub review: TransactionReview,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<SubmitTransactionRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoring_result: Option<ScoringResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub travel_rule_info: Option<TravelRuleInfo>,
}

/// Represents the scoring result of a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScoringResult {
    pub score: f64,
    pub dry_score: f64,
    pub matched_rules: Vec<MatchedRule>,
    pub action: String,
    pub rule_cnt: u32,
    pub dry_run_rule_cnt: u32,
}

/// Represents a matched rule.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MatchedRule {
    pub id: String,
    pub name: String,
    pub revision: u32,
    pub title: String,
    pub score: f64,
    pub dry_run: bool,
    pub action: String,
}

/// Represents the Travel Rule information of a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TravelRuleInfo {
    pub protocol_name: String,
    pub applicant: TravelRuleParticipant,
    pub counterparty: TravelRuleParticipant,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applicant_vasp_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterparty_vasp_id: Option<String>,
}

/// Represents a participant in a Travel Rule transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TravelRuleParticipant {
    pub external_user_id: String,
    pub full_name: String,
}

/// Represents the review status of a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReview {
    pub review_id: String,
    pub attempt_id: String,
    pub attempt_cnt: u32,
    pub level_name: String,
    pub create_date: String,
    pub review_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_result: Option<TransactionReviewResult>,
}

/// Represents the result of a transaction review.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReviewResult {
    pub review_answer: String,
}

/// Represents the response from deleting a transaction.
#[derive(Deserialize, Debug)]
pub struct DeleteTransactionResponse {
    pub deleted: u32,
}

/// Represents a single transaction in a bulk import request.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BulkTransactionImportRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applicant_id: Option<String>,
    pub data: SubmitTransactionRequest,
}

/// Represents the response from a bulk transaction import.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BulkTransactionImportResponse {
    pub created_cnt: u32,
}
