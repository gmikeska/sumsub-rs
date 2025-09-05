// src/misc.rs

//! This module will contain data structures for miscellaneous endpoints.

use serde::Deserialize;

/// Represents an audit trail event.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuditTrailEvent {
    pub ts: String,
    pub client_id: String,
    pub activity: String,
    pub subject_name: String,
    pub ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(rename = "xClientId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_client_id: Option<String>,
    pub correlation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applicant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

use serde::Serialize;
use crate::actions::RequiredIdDocs;

/// Represents the health status of the API.
#[derive(Deserialize, Debug)]
pub struct ApiHealthStatus {
    pub status: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateAccessTokenRequest<'a> {
    pub level_name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_user_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_in_secs: Option<u64>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateWebsdkLinkRequest<'a> {
    pub level_name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_user_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_in_secs: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct GenerateWebsdkLinkResponse {
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewApplicantAccessTokenResponse {
    pub token: String,
    pub user_id: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendVerificationMessageRequest<'a> {
    pub lang: &'a str,
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AvailableLevel {
    pub name: String,
    pub title: String,
    pub review_strategy: String,
    pub required_id_docs: RequiredIdDocs,
}
