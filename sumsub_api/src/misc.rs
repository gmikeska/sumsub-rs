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

/// Represents the health status of the API.
#[derive(Deserialize, Debug)]
pub struct ApiHealthStatus {
    // This endpoint returns an empty body on success, so this struct is empty.
    // The success of the call is determined by the HTTP status code.
}
