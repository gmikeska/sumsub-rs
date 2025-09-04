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
}

impl ToString for CheckType {
    fn to_string(&self) -> String {
        match self {
            CheckType::Poa => "POA".to_string(),
            CheckType::SimilarSearch => "SIMILAR_SEARCH".to_string(),
            CheckType::Tin => "TIN".to_string(),
            CheckType::Company => "COMPANY".to_string(),
        }
    }
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
