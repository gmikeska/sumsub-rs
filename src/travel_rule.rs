use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitiateSdkRequest {
    pub txn_id: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitiateSdkResponse {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchTransactionRequest {
    pub txn_chain_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OwnershipStatus {
    Confirmed,
    Rejected,
}

impl ToString for OwnershipStatus {
    fn to_string(&self) -> String {
        match self {
            OwnershipStatus::Confirmed => "confirmed".to_string(),
            OwnershipStatus::Rejected => "rejected".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmWalletOwnershipRequest {
    pub public_key: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportWalletAddressesRequest {
    pub address: String,
    pub currency: String,
    pub network: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportWalletAddressesResponse {
    pub imported: u32,
    #[serde(rename = "notImported")]
    pub not_imported: u32,
    pub failed: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetTransactionBlockRequest {
    pub reason: String,
    pub control: String,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWalletAddressRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct VaspsResponse {
    pub list: VaspList,
}

#[derive(Deserialize, Debug)]
pub struct VaspList {
    pub items: Vec<Vasp>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vasp {
    pub id: String,
    pub name: String,
    pub website: String,
    pub logo: String,
    pub is_test: bool,
}
