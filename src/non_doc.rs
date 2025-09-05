// src/non_doc.rs

//! This module will contain the data structures for the "Non-Doc Verification" section of the Sumsub API.

use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmNonDocDataRequest<'a> {
    pub consent: &'a str,
}
