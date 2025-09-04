// src/error.rs

//! This module defines the custom error types used throughout the crate.

use thiserror::Error;

/// The error type for the Sumsub API client.
#[derive(Error, Debug)]
pub enum SumsubError {
    /// An error occurred while making a request with `reqwest`.
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// An error occurred during JSON serialization or deserialization.
    #[error("Serde JSON error: {0}")]
    Serde(#[from] serde_json::Error),
}
