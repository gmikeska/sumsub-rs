// src/device_intelligence.rs

//! This module will contain the data structures for the "Device Intelligence" section of the Sumsub API.

use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlatformEvent<'a> {
    pub event_type: &'a str,
    pub event_timestamp: &'a str,
    pub correlation_id: &'a str,
    pub device: DeviceFingerprint<'a>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceFingerprint<'a> {
    pub fingerprint: &'a str,
}
