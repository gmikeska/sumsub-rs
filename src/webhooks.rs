// src/webhooks.rs

//! This module contains functionality for handling Sumsub webhooks, including
//! signature verification and payload deserialization.

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

/// Verifies a webhook signature.
///
/// The signature is expected to be a hex-encoded HMAC-SHA1 digest of the request body.
/// This function performs a constant-time comparison to prevent timing attacks.
///
/// # Arguments
///
/// * `secret_key` - The secret key for your webhook.
/// * `payload` - The raw payload of the webhook request.
/// * `signature` - The value of the `X-Payload-Digest` header (hex-encoded).
///
/// # Returns
///
/// `Ok(())` if the signature is valid, `Err` otherwise.
pub fn verify_signature(secret_key: &str, payload: &[u8], signature: &str) -> Result<(), &'static str> {
    let decoded_signature = hex::decode(signature).map_err(|_| "Invalid hex in signature")?;

    let mut mac = HmacSha1::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload);

    mac.verify_slice(&decoded_signature).map_err(|_| "Invalid signature")
}

/// Represents the different types of webhook payloads.
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WebhookPayload {
    ApplicantReviewed(ApplicantReviewedPayload),
    ApplicantPending(ApplicantPendingPayload),
    // TODO: Add other webhook event types as needed.
}

/// Payload for the `applicantReviewed` webhook.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantReviewedPayload {
    pub applicant_id: String,
    pub inspection_id: String,
    pub correlation_id: String,
    pub level_name: String,
    pub external_user_id: Option<String>,
    pub review: WebhookReview,
    pub created_at: String,
    pub applicant_type: String,
}

/// Payload for the `applicantPending` webhook.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantPendingPayload {
    pub applicant_id: String,
    pub inspection_id: String,
    pub correlation_id: String,
    pub level_name: String,
    pub external_user_id: Option<String>,
    pub created_at: String,
}

/// Represents the review section of a webhook payload.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebhookReview {
    pub review_id: String,
    pub attempt_id: String,
    pub attempt_cnt: i32,
    pub elapsed_since_pending_ms: i64,
    pub create_date: String,
    pub review_status: String,
    pub review_result: Option<WebhookReviewResult>,
}

/// Represents the review result section of a webhook payload.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebhookReviewResult {
    pub review_answer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_comment: Option<String>,
    #[serde(rename = "rRejectType", skip_serializing_if = "Option::is_none")]
    pub review_reject_type: Option<String>,
}
