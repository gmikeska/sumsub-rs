// src/actions.rs

//! This module will contain the data structures for applicant actions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the request to create a new applicant action.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicantActionRequest {
    pub external_action_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_source: Option<PaymentSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questionnaires: Option<Vec<Questionnaire>>,
}

/// Represents a payment source for an applicant action.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSource {
    pub fixed_info: PaymentSourceFixedInfo,
}

/// Represents the fixed info for a payment source.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSourceFixedInfo {
    #[serde(rename = "type")]
    pub payment_type: String,
    pub institution_name: String,
    pub full_name: String,
    pub account_identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Represents a questionnaire for an applicant action.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Questionnaire {
    pub id: String,
    pub sections: HashMap<String, Section>,
}

/// Represents a section in a questionnaire.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub items: HashMap<String, Item>,
}

/// Represents an item in a questionnaire section.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
}

/// Represents an applicant action.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantAction {
    pub id: String,
    pub created_at: String,
    pub client_id: String,
    pub external_action_id: String,
    pub applicant_id: String,
    #[serde(rename = "type")]
    pub action_type: String,
    pub review: ActionReview,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<Vec<Check>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_id_docs: Option<RequiredIdDocs>,
}

/// Represents a check performed within an action.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub answer: String,
    pub check_type: String,
    pub created_at: String,
    pub id: String,
    pub attempt_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Represents the required documents for an action.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequiredIdDocs {
    pub doc_sets: Vec<DocSet>,
}

/// Represents a document set.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocSet {
    pub id_doc_set_type: String,
    pub types: Vec<String>,
}


/// Represents the review status of an applicant action.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionReview {
    pub review_id: String,
    pub attempt_id: String,
    pub attempt_cnt: u32,
    pub level_name: String,
    pub create_date: String,
    pub review_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_result: Option<ReviewResult>,
}

/// Represents the result of a review.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResult {
    pub review_answer: String,
}

/// Represents the response from a request to check an action.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestActionCheckResponse {
    pub id: String,
    pub created_at: String,
    pub client_id: String,
    pub external_action_id: String,
    pub applicant_id: String,
    pub review: ActionReview,
}

/// Represents the response from a request to get a list of applicant actions.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicantActionsResponse {
    pub items: Vec<ApplicantAction>,
    pub total_items: u32,
}
