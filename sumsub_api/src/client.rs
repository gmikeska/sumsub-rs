// src/client.rs

//! This module contains the main `Client` struct, which is used to interact
//! with the Sumsub API. It handles request signing and sending requests to the
//! API endpoints.

use hmac::{Hmac, Mac};
use reqwest::Method;
use serde::Serialize;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::SumsubError;
use crate::models::{Applicant, CreateApplicantRequest, FixedInfo};
use crate::misc::{ApiHealthStatus, AuditTrailEvent};
use crate::actions::{ApplicantAction, CreateApplicantActionRequest, GetApplicantActionsResponse, Questionnaire, RequestActionCheckResponse};
use crate::kyb::{CompanyInfo, GetAdditionalCompanyCheckDataResponse, LinkBeneficiaryRequest};
use crate::transactions::{BulkTransactionImportRequest, BulkTransactionImportResponse, DeleteTransactionResponse, SubmitTransactionRequest, SubmitTransactionResponse};
use crate::travel_rule::{ConfirmWalletOwnershipRequest, ImportWalletAddressesRequest, ImportWalletAddressesResponse, InitiateSdkRequest, InitiateSdkResponse, OwnershipStatus, PatchTransactionRequest, SetTransactionBlockRequest};

type HmacSha256 = Hmac<Sha256>;

const BASE_URL: &str = "https://api.sumsub.com";

/// Signs a request to the Sumsub API.
///
/// This is a private function that generates the `X-App-Access-Sig` header
/// value.
///
/// # Arguments
///
/// * `secret_key` - The secret key for the app token.
/// * `ts` - The timestamp of the request.
/// * `method` - The HTTP method of the request (e.g., "POST").
/// * `path` - The path of the request, including the query string.
/// * `body` - The body of the request.
///
/// # Returns
///
/// A hex-encoded signature.
fn sign_request(
    secret_key: &str,
    ts: u64,
    method: &str,
    path: &str,
    body: &Option<String>,
) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(ts.to_string().as_bytes());
    mac.update(method.as_bytes());
    mac.update(path.as_bytes());
    if let Some(body) = body {
        mac.update(body.as_bytes());
    }

    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    hex::encode(code_bytes)
}

/// A client for the Sumsub API.
#[derive(Debug)]
pub struct Client {
    app_token: String,
    secret_key: String,
    http_client: reqwest::Client,
}

impl Client {
    /// Creates a new `Client`.
    ///
    /// # Arguments
    ///
    /// * `app_token` - The app token for your Sumsub application.
    /// * `secret_key` - The secret key for your Sumsub application.
    ///
    /// # Example
    ///
    /// ```
    /// use sumsub_api::client::Client;
    ///
    /// let client = Client::new("YOUR_APP_TOKEN".to_string(), "YOUR_SECRET_KEY".to_string());
    /// ```
    pub fn new(app_token: String, secret_key: String) -> Self {
        Self {
            app_token,
            secret_key,
            http_client: reqwest::Client::new(),
        }
    }

    /// Sends a request to the Sumsub API.
    ///
    /// This is a private helper function that handles the common logic for
    /// sending requests to the API, including signing the request and
    /// adding the required headers.
    async fn send_request<T: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<T>,
    ) -> Result<reqwest::Response, SumsubError> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let body_str = if let Some(body) = body {
            Some(serde_json::to_string(&body).map_err(SumsubError::from)?)
        } else {
            None
        };

        let signature = sign_request(
            &self.secret_key,
            ts,
            method.as_str(),
            path,
            &body_str,
        );

        let url = format!("{}{}", BASE_URL, path);
        let mut request_builder = self.http_client.request(method, &url);

        request_builder = request_builder
            .header("X-App-Token", &self.app_token)
            .header("X-App-Access-Sig", signature)
            .header("X-App-Access-Ts", ts.to_string());

        if let Some(body) = body_str {
            request_builder = request_builder
                .header("Content-Type", "application/json")
                .body(body);
        }

        request_builder.send().await.map_err(SumsubError::from)
    }

    /// Creates a new applicant.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/create-applicant)
    ///
    /// # Arguments
    ///
    /// * `request` - The request to create an applicant.
    /// * `level_name` - The name of the verification level to assign to the applicant.
    pub async fn create_applicant(
        &self,
        request: CreateApplicantRequest,
        level_name: &str,
    ) -> Result<Applicant, SumsubError> {
        let path = format!("/resources/applicants?levelName={}", level_name);
        let response = self
            .send_request(Method::POST, &path, Some(request))
            .await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Gets applicant data.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/get-applicant-data)
    ///
    /// # Arguments
    ///
    /// * `applicant_id` - The ID of the applicant to get.
    pub async fn get_applicant_data(
        &self,
        applicant_id: &str,
    ) -> Result<Applicant, SumsubError> {
        let path = format!("/resources/applicants/{}/one", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Gets audit trail events.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/audit-trail-events)
    pub async fn get_audit_trail_events(&self) -> Result<Vec<AuditTrailEvent>, SumsubError> {
        let path = "/resources/auditTrailEvents/";
        let response = self.send_request(Method::GET, path, None::<()>).await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Gets the API health status.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/review-api-health)
    pub async fn get_api_health_status(&self) -> Result<ApiHealthStatus, SumsubError> {
        let path = "/resources/status/api";
        let response = self.send_request(Method::GET, path, None::<()>).await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Creates a new applicant action.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/create-applicant-action)
    ///
    /// # Arguments
    ///
    /// * `applicant_id` - The ID of the applicant to create the action for.
    /// * `level_name` - The name of the verification level to assign to the action.
    /// * `request` - The request to create an applicant action.
    pub async fn create_applicant_action(
        &self,
        applicant_id: &str,
        level_name: &str,
        request: CreateApplicantActionRequest,
    ) -> Result<ApplicantAction, SumsubError> {
        let path = format!(
            "/resources/applicantActions/-/forApplicant/{}?levelName={}",
            applicant_id, level_name
        );
        let response = self
            .send_request(Method::POST, &path, Some(request))
            .await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Requests a check for an applicant action.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/request-action-check)
    ///
    /// # Arguments
    ///
    /// * `action_id` - The ID of the action to check.
    pub async fn request_action_check(
        &self,
        action_id: &str,
    ) -> Result<RequestActionCheckResponse, SumsubError> {
        let path = format!(
            "/resources/applicantActions/{}/review/status/pending",
            action_id
        );
        let response = self.send_request(Method::POST, &path, None::<()>).await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Gets a list of applicant actions.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/get-applicant-actions)
    ///
    /// # Arguments
    ///
    /// * `applicant_id` - The ID of the applicant to get the actions for.
    pub async fn get_applicant_actions(
        &self,
        applicant_id: &str,
    ) -> Result<GetApplicantActionsResponse, SumsubError> {
        let path = format!("/resources/applicantActions/-;applicantId={}", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        let result: GetApplicantActionsResponse = response.json().await.map_err(SumsubError::from)?;
        Ok(result)
    }

    /// Gets information about a specific applicant action.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/get-action-information)
    ///
    /// # Arguments
    ///
    /// * `action_id` - The ID of the action to get information for.
    pub async fn get_action_information(
        &self,
        action_id: &str,
    ) -> Result<ApplicantAction, SumsubError> {
        let path = format!("/resources/applicantActions/{}/one", action_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Adds a questionnaire to an applicant action.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/add-applicant-action-questionnaire)
    ///
    /// # Arguments
    ///
    /// * `action_id` - The ID of the action to add the questionnaire to.
    /// * `questionnaire` - The questionnaire to add.
    pub async fn add_applicant_action_questionnaire(
        &self,
        action_id: &str,
        questionnaire: Questionnaire,
    ) -> Result<Questionnaire, SumsubError> {
        let path = format!(
            "/resources/applicantActions/{}/questionnaires",
            action_id
        );
        let response = self
            .send_request(Method::POST, &path, Some(questionnaire))
            .await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Links a beneficiary to a company.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/link-beneficiary-to-company-kyb-20)
    ///
    /// # Arguments
    ///
    /// * `applicant_id` - The ID of the company applicant.
    /// * `request` - The request to link a beneficiary.
    pub async fn link_beneficiary(
        &self,
        applicant_id: &str,
        request: LinkBeneficiaryRequest,
    ) -> Result<(), SumsubError> {
        let path = format!(
            "/resources/applicants/{}/fixedInfo/companyInfo/beneficiaries",
            applicant_id
        );
        self.send_request(Method::POST, &path, Some(request)).await?;
        Ok(())
    }

    /// Unlinks a beneficiary from a company.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/unlink-beneficiary-from-company-kyb-20)
    ///
    /// # Arguments
    ///
    /// * `applicant_id` - The ID of the company applicant.
    /// * `beneficiary_id` - The ID of the beneficiary to unlink.
    pub async fn unlink_beneficiary(
        &self,
        applicant_id: &str,
        beneficiary_id: &str,
    ) -> Result<(), SumsubError> {
        let path = format!(
            "/resources/applicants/{}/fixedInfo/companyInfo/beneficiaries/{}",
            applicant_id, beneficiary_id
        );
        self.send_request(Method::DELETE, &path, None::<()>).await?;
        Ok(())
    }

    /// Changes the extracted company data.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/change-extracted-company-data)
    ///
    /// # Arguments
    ///
    /// * `applicant_id` - The ID of the company applicant.
    /// * `company_info` - The company data to update.
    pub async fn change_extracted_company_data(
        &self,
        applicant_id: &str,
        company_info: CompanyInfo,
    ) -> Result<CompanyInfo, SumsubError> {
        let path = format!("/resources/applicants/{}/info/companyInfo", applicant_id);
        let response = self
            .send_request(Method::PATCH, &path, Some(company_info))
            .await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Changes the provided company data.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/change-provided-info-fixedinfo)
    ///
    /// # Arguments
    ///
    /// * `applicant_id` - The ID of the company applicant.
    /// * `fixed_info` - The company data to update.
    pub async fn change_provided_company_data(
        &self,
        applicant_id: &str,
        fixed_info: FixedInfo,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/fixedInfo", applicant_id);
        self.send_request(Method::PATCH, &path, Some(fixed_info)).await?;
        Ok(())
    }

    /// Gets additional company check data.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/get-additional-company-check-data)
    ///
    /// # Arguments
    ///
    /// * `applicant_id` - The ID of the company applicant.
    pub async fn get_additional_company_check_data(
        &self,
        applicant_id: &str,
    ) -> Result<GetAdditionalCompanyCheckDataResponse, SumsubError> {
        let path = format!(
            "/resources/checks/latest?type=COMPANY&applicantId={}",
            applicant_id
        );
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Submits a transaction for an existing applicant.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/submit-transaction-for-existing-applicant)
    ///
    /// # Arguments
    ///
    /// * `applicant_id` - The ID of the applicant to submit the transaction for.
    /// * `request` - The transaction to submit.
    pub async fn submit_transaction(
        &self,
        applicant_id: &str,
        request: SubmitTransactionRequest,
    ) -> Result<SubmitTransactionResponse, SumsubError> {
        let path = format!(
            "/resources/applicants/{}/kyt/txns/-/data",
            applicant_id
        );
        let response = self
            .send_request(Method::POST, &path, Some(request))
            .await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Submits a transaction for a non-existing applicant.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/submit-transaction-for-non-existing-applicant)
    ///
    /// # Arguments
    ///
    /// * `request` - The transaction to submit.
    pub async fn submit_transaction_for_non_existing_applicant(
        &self,
        request: SubmitTransactionRequest,
    ) -> Result<SubmitTransactionResponse, SumsubError> {
        let path = "/resources/applicants/-/kyt/txns/-/data";
        let response = self
            .send_request(Method::POST, &path, Some(request))
            .await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Deletes a transaction.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/delete-transaction)
    ///
    /// # Arguments
    ///
    /// * `txn_id` - The ID of the transaction to delete.
    pub async fn delete_transaction(
        &self,
        txn_id: &str,
    ) -> Result<DeleteTransactionResponse, SumsubError> {
        let path = format!("/resources/kyt/txns/{}", txn_id);
        let response = self.send_request(Method::DELETE, &path, None::<()>).await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Imports transactions in bulk.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/bulk-transaction-import)
    ///
    /// # Arguments
    ///
    /// * `requests` - A vector of transactions to import.
    pub async fn bulk_transaction_import(
        &self,
        requests: Vec<BulkTransactionImportRequest>,
    ) -> Result<BulkTransactionImportResponse, SumsubError> {
        let path = "/resources/kyt/misc/txns/import";
        let body = requests
            .into_iter()
            .map(|r| serde_json::to_string(&r))
            .collect::<Result<Vec<String>, _>>()
            .map_err(SumsubError::from)?
            .join("\n");

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = sign_request(
            &self.secret_key,
            ts,
            "POST",
            path,
            &Some(body.clone()),
        );

        let url = format!("{}{}", BASE_URL, path);
        let mut request_builder = self.http_client.request(Method::POST, &url);

        request_builder = request_builder
            .header("X-App-Token", &self.app_token)
            .header("X-App-Access-Sig", signature)
            .header("X-App-Access-Ts", ts.to_string())
            .header("Content-Type", "application/x-ndjson")
            .body(body);

        let response = request_builder.send().await.map_err(SumsubError::from)?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Initiates the SDK for a Travel Rule transaction.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/initiate-sdk-for-travel-rule-transaction)
    ///
    /// # Arguments
    ///
    /// * `request` - The request to initiate the SDK.
    pub async fn initiate_sdk_for_travel_rule_transaction(
        &self,
        request: InitiateSdkRequest,
    ) -> Result<InitiateSdkResponse, SumsubError> {
        let path = "/resources/tr/sdk/init";
        let response = self
            .send_request(Method::POST, &path, Some(request))
            .await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Patches a transaction with a chain transaction ID.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/patch-transaction-with-chain-txid)
    ///
    /// # Arguments
    ///
    /// * `txn_id` - The ID of the transaction to patch.
    /// * `request` - The request to patch the transaction.
    pub async fn patch_transaction_with_chain_transaction_id(
        &self,
        txn_id: &str,
        request: PatchTransactionRequest,
    ) -> Result<SubmitTransactionResponse, SumsubError> {
        let path = format!("/resources/kyt/txns/{}/data/info", txn_id);
        let response = self
            .send_request(Method::PATCH, &path, Some(request))
            .await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Confirms or rejects ownership of a transaction.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/confirm-or-reject-transaction-ownership)
    ///
    /// # Arguments
    ///
    /// * `txn_id` - The ID of the transaction.
    /// * `status` - The ownership status to set.
    pub async fn confirm_or_reject_transaction_ownership(
        &self,
        txn_id: &str,
        status: OwnershipStatus,
    ) -> Result<SubmitTransactionResponse, SumsubError> {
        let path = format!(
            "/resources/kyt/txns/{}/ownership/{}",
            txn_id,
            status.to_string()
        );
        let response = self.send_request(Method::POST, &path, None::<()>).await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Confirms wallet ownership.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/confirm-wallet-ownership)
    ///
    /// # Arguments
    ///
    /// * `txn_id` - The ID of the transaction.
    /// * `request` - The request to confirm wallet ownership.
    pub async fn confirm_wallet_ownership(
        &self,
        txn_id: &str,
        request: ConfirmWalletOwnershipRequest,
    ) -> Result<SubmitTransactionResponse, SumsubError> {
        let path = format!("/resources/kyt/txns/{}/travelRuleOwnership", txn_id);
        let response = self
            .send_request(Method::POST, &path, Some(request))
            .await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Imports wallet addresses in bulk.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/import-wallet-addresses)
    ///
    /// # Arguments
    ///
    /// * `requests` - A vector of wallet addresses to import.
    pub async fn import_wallet_addresses(
        &self,
        requests: Vec<ImportWalletAddressesRequest>,
    ) -> Result<ImportWalletAddressesResponse, SumsubError> {
        let path = "/resources/kyt/txns/-/importAddress";
        let body = requests
            .into_iter()
            .map(|r| serde_json::to_string(&r))
            .collect::<Result<Vec<String>, _>>()
            .map_err(SumsubError::from)?
            .join("\n");

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = sign_request(
            &self.secret_key,
            ts,
            "POST",
            path,
            &Some(body.clone()),
        );

        let url = format!("{}{}", BASE_URL, path);
        let mut request_builder = self.http_client.request(Method::POST, &url);

        request_builder = request_builder
            .header("X-App-Token", &self.app_token)
            .header("X-App-Access-Sig", signature)
            .header("X-App-Access-Ts", ts.to_string())
            .header("Content-Type", "application/x-ndjson")
            .body(body);

        let response = request_builder.send().await.map_err(SumsubError::from)?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Gets transaction data.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/get-transaction-data)
    ///
    /// # Arguments
    ///
    /// * `txn_id` - The ID of the transaction to get.
    pub async fn get_transaction_data(
        &self,
        txn_id: &str,
    ) -> Result<SubmitTransactionResponse, SumsubError> {
        let path = format!("/resources/kyt/txns/{}", txn_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        response.json().await.map_err(SumsubError::from)
    }

    /// Gets all transactions for an applicant.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/get-all-transactions-for-applicant)
    ///
    /// # Arguments
    ///
    /// * `applicant_id` - The ID of the applicant.
    pub async fn get_all_transactions_for_applicant(
        &self,
        applicant_id: &str,
    ) -> Result<Vec<SubmitTransactionResponse>, SumsubError> {
        let path = format!("/resources/kyt/txns?applicantId={}", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;

        #[derive(serde::Deserialize)]
        struct TransactionList {
            list: Vec<SubmitTransactionResponse>,
        }
        let list: TransactionList = response.json().await.map_err(SumsubError::from)?;
        Ok(list.list)
    }

    /// Sets the block status for a transaction.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/set-transaction-block)
    ///
    /// # Arguments
    ///
    /// * `txn_id` - The ID of the transaction.
    /// * `request` - The request to set the block status.
    pub async fn set_transaction_block(
        &self,
        txn_id: &str,
        request: SetTransactionBlockRequest,
    ) -> Result<SubmitTransactionResponse, SumsubError> {
        let path = format!("/resources/kyt/txns/{}/block", txn_id);
        let response = self
            .send_request(Method::POST, &path, Some(request))
            .await?;
        response.json().await.map_err(SumsubError::from)
    }
}
