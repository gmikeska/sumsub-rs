// src/models.rs

//! This module contains the data structures used for API requests and
//! responses. These structs are used for serialization and deserialization
//! of JSON data.

use serde::{Deserialize, Serialize};
use crate::kyb::CompanyInfo;

/// Represents the request to create a new applicant.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicantRequest {
    /// A unique identifier for the applicant in your system.
    pub external_user_id: String,
    /// The applicant's email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The applicant's phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// The applicant's fixed information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_info: Option<FixedInfo>,
    /// The type of applicant to create.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applicant_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Info>,
}

/// Represents the fixed information about an applicant.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FixedInfo {
    /// The applicant's country of residence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// The applicant's first name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// The applicant's last name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// The applicant's date of birth in `YYYY-MM-DD` format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dob: Option<String>,
    /// The applicant's place of birth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_of_birth: Option<String>,
    /// The company's information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_info: Option<CompanyInfo>,
}

/// Represents the information about an applicant.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dob: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_of_birth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_of_birth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_of_birth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nationality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<Address>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_info: Option<CompanyInfo>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Address {
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


/// Represents a Sumsub applicant.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Applicant {
    /// The unique identifier of the applicant.
    pub id: String,
    /// The timestamp of when the applicant was created.
    pub created_at: String,
    /// The ID of the client that the applicant belongs to.
    pub client_id: String,
    /// The ID of the inspection associated with the applicant.
    pub inspection_id: String,
    /// The external user ID of the applicant.
    pub external_user_id: String,
    /// The applicant's email address.
    pub email: Option<String>,
    /// The applicant's phone number.
    pub phone: Option<String>,
    /// The platform from which the applicant was created.
    pub applicant_platform: Option<String>,
    /// The review status of the applicant.
    pub review: Review,
    /// The type of the applicant (e.g., "individual" or "company").
    #[serde(rename = "type")]
    pub applicant_type: String,
}

/// Represents the review status of an applicant.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    /// The review status of the applicant (e.g., "completed", "pending").
    pub review_status: String,
}
