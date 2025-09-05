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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddDocumentMetadata<'a> {
    pub id_doc_type: &'a str,
    pub country: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dob: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_of_birth: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued_date: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_sub_type: Option<&'a str>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimulateReviewRequest<'a> {
    pub review_answer: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_labels: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_reject_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_comment: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_comment: Option<&'a str>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddConsentsRequest<'a> {
    pub accepted: Vec<&'a str>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantFacingConsentsResponse {
    pub consents: Vec<ApplicantFacingConsent>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantFacingConsent {
    pub id: String,
    #[serde(rename = "type")]
    pub consent_type: String,
    pub required: bool,
    pub url: String,
    pub order_index: i32,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    pub level_name: String,
    pub title: String,
    pub description: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub created_at: String,
    pub applicant_id: String,
    pub agent: Agent,
    pub note: String,
    pub attachments: Vec<Attachment>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub client_id: String,
    pub email: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub file_name: String,
    pub created_at: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddNoteRequest<'a> {
    pub note: &'a str,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EditNoteRequest<'a> {
    pub note: &'a str,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VerificationStepStatus {
    pub review_answer: String,
    pub check_type: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHistoryRecord {
    pub created_at: String,
    pub status: String,
    pub review_answer: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub image_id: String,
    pub inspection_id: String,
    pub id_doc_type: String,
    pub added_at: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AmlData {
    pub applicant: AmlApplicant,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AmlApplicant {
    pub id: String,
    pub inspection_id: String,
    pub external_applicant_id: Option<String>,
    pub source_key: Option<String>,
    pub created_at_ms: u64,
    pub info: AmlApplicantInfo,
    pub hits: Vec<AmlHit>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AmlApplicantInfo {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dob: Option<String>,
    pub country: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AmlHit {
    pub id: String,
    pub hit_id_in_source: String,
    pub source_name: String,
    pub created_at_ms: u64,
    pub review: AmlReview,
    pub match_info: AmlMatchInfo,
    pub data: serde_json::Value,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AmlReview {
    pub status: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AmlMatchInfo {
    pub match_types: Vec<String>,
    pub match_strength: f64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAmlHitReviewRequest<'a> {
    pub review_status: &'a str,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeactivateApplicantRequest<'a> {
    pub review: DeactivateApplicantReview<'a>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeactivateApplicantReview<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_comment: Option<&'a str>,
}
