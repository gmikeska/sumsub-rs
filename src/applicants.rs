// src/applicants.rs

//! This module will contain the data structures for the "Applicants" section of the Sumsub API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::actions::Questionnaire;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantStatus {
    pub create_date: String,
    pub review_date: Option<String>,
    pub start_date: Option<String>,
    pub review_result: Option<ReviewResult>,
    pub review_status: String,
    pub moderation_comment: Option<String>,
    pub client_comment: Option<String>,
    pub reject_labels: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResult {
    pub review_answer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_reject_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_labels: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModerationState {
    pub created_at: String,
    pub client_id: String,
    pub inspection_id: String,
    pub applicant_id: String,
    pub level_name: String,
    pub external_user_id: Option<String>,
    pub info: Option<serde_json::Value>, // Can be complex, using Value for now
    pub moderation: Option<ModerationDetails>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModerationDetails {
    pub id: String,
    pub state: i32,
    pub method: String,
    pub user_name: String,
    pub comment: Option<String>,
    pub time: String,
    pub pretty_time: String,
    pub is_auto: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlacklistRequest {
    pub note: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShareTokenRequest<'a> {
    pub level_name: &'a str,
    pub external_user_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_in_secs: Option<u64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShareTokenResponse {
    pub token: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportApplicantRequest<'a> {
    pub token: &'a str,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportApplicantResponse {
    pub applicant_id: String,
    pub inspection_id: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IngestCompletedRequest {
    pub applicant: IngestApplicant,
    pub review: IngestReview,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_sets: Option<Vec<IngestDocSet>>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IngestApplicant {
    pub external_user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<crate::models::Info>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IngestReview {
    pub level_name: String,
    pub review_answer: String, // "GREEN" or "RED"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_comment: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IngestDocSet {
    pub id_doc_set_type: String,
    pub fields: HashMap<String, String>,
}


#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApplicantRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questionnaires: Option<Vec<Questionnaire>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimilarByTextAndFaceResult {
    pub matches: Vec<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantEvent {
    pub created_at: String,
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangeApplicantDataRequest {
    pub info: crate::models::Info,
}
