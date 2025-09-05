// src/checks.rs

//! This module will contain the data structures for the "Checks" section of the Sumsub API.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum CheckType {
    #[serde(rename = "POA")]
    Poa,
    #[serde(rename = "SIMILAR_SEARCH")]
    SimilarSearch,
    #[serde(rename = "TIN")]
    Tin,
    #[serde(rename = "COMPANY")]
    Company,
    #[serde(rename = "BANK_CARD")]
    BankCard,
    #[serde(rename = "EMAIL_CONFIRMATION")]
    EmailConfirmation,
    #[serde(rename = "PHONE_CONFIRMATION")]
    PhoneConfirmation,
    #[serde(rename = "IP_CHECK")]
    IpCheck,
    #[serde(rename = "NFC")]
    Nfc,
}

impl ToString for CheckType {
    fn to_string(&self) -> String {
        match self {
            CheckType::Poa => "POA".to_string(),
            CheckType::SimilarSearch => "SIMILAR_SEARCH".to_string(),
            CheckType::Tin => "TIN".to_string(),
            CheckType::Company => "COMPANY".to_string(),
            CheckType::BankCard => "BANK_CARD".to_string(),
            CheckType::EmailConfirmation => "EMAIL_CONFIRMATION".to_string(),
            CheckType::PhoneConfirmation => "PHONE_CONFIRMATION".to_string(),
            CheckType::IpCheck => "IP_CHECK".to_string(),
            CheckType::Nfc => "NFC".to_string(),
        }
    }
}

// For GET /resources/checks/latest?type=POA
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PoaCheckResult {
    pub street: Vec<String>,
    pub street_mrz: Vec<String>,
    pub address: Vec<String>,
    pub address_mrz: Vec<String>,
    pub issue_date: Vec<String>,
    pub issue_date_mrz: Vec<String>,
    pub name: Vec<String>,
    pub name_mrz: Vec<String>,
    pub post_code: Vec<String>,
    pub post_code_mrz: Vec<String>,
    pub town: Vec<String>,
    pub town_mrz: Vec<String>,
    pub name_score: f64,
    pub address_score: f64,
    pub faces: Vec<Face>,
    pub qr: Option<String>,
    pub barcodes: Vec<String>,
    pub doc_quality: DocQuality,
}

#[derive(Deserialize, Debug)]
pub struct Face {
    pub l: i32,
    pub t: i32,
    pub r: i32,
    pub b: i32,
}

#[derive(Deserialize, Debug)]
pub struct DocQuality {
    pub score: f64,
    pub metrics: DocQualityMetrics,
}

#[derive(Deserialize, Debug)]
pub struct DocQualityMetrics {
    pub blur: f64,
    pub dark: f64,
}

// For GET /resources/checks/latest?type=BANK_CARD
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BankCardCheckResult {
    pub applicant_id: String,
    pub card_holder: String,
    pub card_number_mask: String,
}

// For GET /resources/checks/latest?type=EMAIL_CONFIRMATION
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EmailConfirmationCheckResult {
    pub applicant_id: String,
    pub email: String,
    pub confirmed: bool,
}

// For GET /resources/checks/latest?type=PHONE_CONFIRMATION
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PhoneConfirmationCheckResult {
    pub applicant_id: String,
    pub phone: String,
    pub confirmed: bool,
}

// For GET /resources/checks/latest?type=IP_CHECK
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IpCheckResult {
    pub applicant_id: String,
    // ... fields based on documentation
}

// For GET /resources/checks/latest?type=NFC
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NfcCheckResult {
    pub applicant_id: String,
    pub phone: String,
    pub confirmed: bool,
}


#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartCheckRequest<'a> {
    pub applicant_id: &'a str,
    pub check_type: CheckType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_info: Option<AddressInfo>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddressInfo {
    pub country: String,
    pub post_code: String,
    pub town: String,
    pub street: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flat_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_number: Option<String>,
}

// For GET /resources/checks/latest?type=SIMILAR_SEARCH
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimilarSearchResult {
    pub applicant_id: String,
    pub similar_applicants: Vec<SimilarApplicant>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimilarApplicant {
    pub id: String,
    pub match_type: String,
    pub review_answer: String,
}

// For GET /resources/checks/latest?type=TIN
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TinCheckResult {
    pub applicant_id: String,
    pub ssn_status: String,
    pub validation_details: Option<String>,
}
