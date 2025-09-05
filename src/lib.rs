// src/lib.rs

//! A Rust crate for interacting with the Sumsub API.
//!
//! This crate provides a client for the Sumsub API, allowing you to
//! perform actions such as creating applicants, uploading documents, and
//! getting verification results.

/// The `client` module contains the main `Client` struct, which is used
/// to make requests to the Sumsub API.
pub mod client;

/// The `error` module defines the custom error types used in this crate.
pub mod error;

/// The `models` module contains the data structures used for API requests
/// and responses.
pub mod models;

/// The `actions` module contains the data structures for applicant actions.
pub mod actions;

/// The `applicants` module contains the data structures for the "Applicants" section of the Sumsub API.
pub mod applicants;

/// The `checks` module contains the data structures for the "Checks" section of the Sumsub API.
pub mod checks;

/// The `kyb` module contains the data structures for business verification (KYB).
pub mod kyb;

/// The `transactions` module contains the data structures for transaction monitoring.
pub mod transactions;

/// The `travel_rule` module contains the data structures for Travel Rule compliance.
pub mod travel_rule;

/// The `misc` module contains data structures for miscellaneous endpoints.
pub mod misc;

/// The `non_doc` module contains data structures for the "Non-Doc Verification" section.
pub mod non_doc;

/// The `device_intelligence` module contains data structures for the "Device Intelligence" section.
pub mod device_intelligence;

/// The `webhooks` module contains functionality for handling Sumsub webhooks.
pub mod webhooks;
