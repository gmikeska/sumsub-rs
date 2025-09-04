// src/kyb.rs

//! This module will contain the data structures for business verification (KYB).

use serde::{Deserialize, Serialize};

/// Represents the information about a company.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInfo {
    pub company_name: String,
    pub registration_number: String,
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incorporated_on: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
}

/// Represents a physical address.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub town: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_street: Option<String>,
}

/// Represents a request to link a beneficiary to a company.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum LinkBeneficiaryRequest {
    Existing(ExistingBeneficiary),
    New(NewBeneficiary),
}

/// Represents an existing beneficiary to be linked to a company.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExistingBeneficiary {
    pub applicant_id: String,
    pub types: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_size: Option<f64>,
}

/// Represents a new beneficiary to be linked to a company.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct NewBeneficiary {
    pub types: Vec<String>,
    pub beneficiary_info: BeneficiaryInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_size: Option<f64>,
}

/// Represents the information about a new beneficiary.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BeneficiaryInfo {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dob: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_residence_country: Option<String>,
}

/// Represents the response from a request to get additional company check data.
#[derive(Deserialize, Debug)]
pub struct GetAdditionalCompanyCheckDataResponse {
    pub checks: Vec<CompanyCheck>,
}

/// Represents a company check.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CompanyCheck {
    pub answer: String,
    pub created_at: String,
    pub company_check_info: CompanyCheckInfo,
}

/// Represents the information from a company check.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CompanyCheckInfo {
    pub company_name: String,
    pub company_number: String,
    pub status: String,
    #[serde(rename = "type")]
    pub company_type: String,
    pub source: String,
    pub source_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub office_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub office_address_structured: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_address_structured: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incorporated_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry_codes: Option<Vec<IndustryCode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternative_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_info: Option<LicenseInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub officers: Option<Vec<Officer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub significant_persons: Option<Vec<SignificantPerson>>,
}

/// Represents an industry code.
#[derive(Deserialize, Debug)]
pub struct IndustryCode {
    pub code: String,
    pub description: String,
}

/// Represents license information.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LicenseInfo {
    pub license_number: String,
    pub issued_date: String,
    pub valid_until: String,
}

/// Represents a company officer.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Officer {
    pub full_name: String,
    #[serde(rename = "type")]
    pub officer_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dob: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nationality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appointed_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Represents a significant person in a company.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignificantPerson {
    pub full_name: String,
    #[serde(rename = "type")]
    pub person_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dob: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nationality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beneficial_ownership_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nature_of_control: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}
