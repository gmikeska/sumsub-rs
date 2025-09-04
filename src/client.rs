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
use crate::applicants::*;
use crate::checks::*;
use crate::misc::*;
use serde::Deserialize;


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

    async fn handle_response_and_deserialize<T: for<'de> serde::Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, SumsubError> {
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(SumsubError::ApiError { status, message });
        }
        response.json().await.map_err(SumsubError::from)
    }

    async fn handle_empty_response(&self, response: reqwest::Response) -> Result<(), SumsubError> {
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(SumsubError::ApiError { status, message });
        }
        Ok(())
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
    }

    /// Retrieves the latest TIN check result for an applicant.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-tin-check-results)
    pub async fn get_latest_tin_check_result(
        &self,
        applicant_id: &str,
    ) -> Result<TinCheckResult, SumsubError> {
        self.get_latest_check_result(applicant_id, CheckType::Tin)
            .await
    }

    /// Retrieves the latest similar search result for an applicant.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-duplicate-applicants-check-result)
    pub async fn get_latest_similar_search_result(
        &self,
        applicant_id: &str,
    ) -> Result<SimilarSearchResult, SumsubError> {
        self.get_latest_check_result(applicant_id, CheckType::SimilarSearch)
            .await
    }

    /// Gets audit trail events.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/audit-trail-events)
    pub async fn get_audit_trail_events(&self) -> Result<Vec<AuditTrailEvent>, SumsubError> {
        let path = "/resources/auditTrailEvents/";
        let response = self.send_request(Method::GET, path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Gets the API health status.
    ///
    /// [Sumsub API reference](https://docs.sumsub.com/reference/review-api-health)
    pub async fn get_api_health_status(&self) -> Result<ApiHealthStatus, SumsubError> {
        let path = "/resources/status/api";
        let response = self.send_request(Method::GET, path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_empty_response(response).await
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
        let response = self.send_request(Method::DELETE, &path, None::<()>).await?;
        self.handle_empty_response(response).await
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
        self.handle_response_and_deserialize(response).await
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
        let response = self.send_request(Method::PATCH, &path, Some(fixed_info)).await?;
        self.handle_empty_response(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        self.handle_response_and_deserialize(response).await
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
        let list: TransactionList = self.handle_response_and_deserialize(response).await?;
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
        self.handle_response_and_deserialize(response).await
    }

    // Applicants Section

    /// Moves an applicant to a different verification level.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#change-level-and-reset-steps)
    pub async fn move_applicant_to_level(
        &self,
        applicant_id: &str,
        level_name: &str,
    ) -> Result<(), SumsubError> {
        let path = format!(
            "/resources/applicants/{}/moveToLevel?levelName={}",
            applicant_id, level_name
        );
        let response = self.send_request(Method::POST, &path, None::<()>).await?;
        self.handle_empty_response(response).await
    }

    /// Updates fixed information for an applicant.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#updating-fixed-applicant-info)
    pub async fn update_applicant_fixed_info(
        &self,
        applicant_id: &str,
        fixed_info: FixedInfo,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/fixedInfo", applicant_id);
        let response = self
            .send_request(Method::PATCH, &path, Some(fixed_info))
            .await?;
        self.handle_empty_response(response).await
    }

    /// Retrieves the review status for an applicant.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#retrieving-review-status)
    pub async fn get_applicant_status(
        &self,
        applicant_id: &str,
    ) -> Result<ApplicantStatus, SumsubError> {
        let path = format!("/resources/applicants/{}/status", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Retrieves moderation states for an applicant to clarify rejections.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#clarify-rejection-reason)
    pub async fn get_applicant_moderation_states(
        &self,
        applicant_id: &str,
    ) -> Result<Vec<ModerationState>, SumsubError> {
        let path = format!("/resources/moderationStates/-;applicantId={}", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Requests a re-check for an applicant.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#request-re-check)
    pub async fn request_applicant_recheck(&self, applicant_id: &str) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/status/pending", applicant_id);
        let response = self.send_request(Method::POST, &path, None::<()>).await?;
        self.handle_empty_response(response).await
    }

    /// Adds an applicant to the blocklist.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#add-to-blocklist)
    pub async fn add_applicant_to_blocklist(
        &self,
        applicant_id: &str,
        note: String,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/blacklist", applicant_id);
        let request = BlacklistRequest { note };
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Creates a share token for an applicant.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#reusable-kyc)
    pub async fn create_share_token<'a>(
        &self,
        request: ShareTokenRequest<'a>,
    ) -> Result<ShareTokenResponse, SumsubError> {
        let path = "/resources/accessTokens/-/shareToken";
        let response = self.send_request(Method::POST, path, Some(request)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Imports a shared applicant.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#reusable-kyc)
    pub async fn import_shared_applicant<'a>(
        &self,
        token: &'a str,
    ) -> Result<ImportApplicantResponse, SumsubError> {
        let path = "/resources/applicants/-/import";
        let request = ImportApplicantRequest { token };
        let response = self.send_request(Method::POST, path, Some(request)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Resets a single verification step for an applicant.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#reset-an-applicants-step)
    pub async fn reset_applicant_step(
        &self,
        applicant_id: &str,
        id_doc_set_type: &str,
    ) -> Result<(), SumsubError> {
        let path = format!(
            "/resources/applicants/{}/resetStep/{}",
            applicant_id, id_doc_set_type
        );
        let response = self.send_request(Method::POST, &path, None::<()>).await?;
        self.handle_empty_response(response).await
    }

    /// Resets an applicant entirely.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#reset-an-applicant)
    pub async fn reset_applicant(&self, applicant_id: &str) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/reset", applicant_id);
        let response = self.send_request(Method::POST, &path, None::<()>).await?;
        self.handle_empty_response(response).await
    }

    /// Ingests a completed applicant profile.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#batch-import-of-completed-applicants)
    pub async fn ingest_completed_applicant(
        &self,
        request: IngestCompletedRequest,
    ) -> Result<(), SumsubError> {
        let path = "/resources/applicants/-/ingestCompleted";
        let response = self.send_request(Method::POST, path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Updates top-level applicant data (e.g., email, phone).
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#updating-top-level-applicant-data)
    pub async fn update_applicant_top_level_data(
        &self,
        applicant_id: &str,
        request: UpdateApplicantRequest,
    ) -> Result<Applicant, SumsubError> {
        let path = format!("/resources/applicants/{}", applicant_id);
        let response = self
            .send_request(Method::PATCH, &path, Some(request))
            .await?;
        self.handle_response_and_deserialize(response).await
    }

    // Checks Section

    /// Starts a specific check for an applicant.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#starting-specific-checks)
    pub async fn start_check(
        &self,
        request: StartCheckRequest<'_>,
    ) -> Result<serde_json::Value, SumsubError> {
        let path = "/resources/checks";
        let response = self.send_request(Method::POST, path, Some(request)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Retrieves the latest check results for an applicant.
    /// The return type `T` must be a struct that can be deserialized from the JSON response for the given `check_type`.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#retrieving-check-results)
    pub async fn get_latest_check_result<T: for<'de> serde::Deserialize<'de>>(
        &self,
        applicant_id: &str,
        check_type: CheckType,
    ) -> Result<T, SumsubError> {
        let path = format!(
            "/resources/checks/latest?type={}&applicantId={}",
            check_type.to_string(),
            applicant_id
        );
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    // Additional/Supplemental Methods

    /// Generates an access token for a new applicant for the WebSDK.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#generate-access-token)
    pub async fn generate_token_for_new_applicant(
        &self,
        level_name: &str,
        external_user_id: Option<&str>,
        ttl_in_secs: Option<u64>,
    ) -> Result<NewApplicantAccessTokenResponse, SumsubError> {
        let mut path = format!("/resources/accessTokens?levelName={}", level_name);
        if let Some(id) = external_user_id {
            path.push_str(&format!("&externalUserId={}", id));
        }
        if let Some(ttl) = ttl_in_secs {
            path.push_str(&format!("&ttlInSecs={}", ttl));
        }
        let response = self.send_request(Method::POST, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Generates an access token for an existing applicant for the WebSDK.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#access-tokens-for-existing-users)
    pub async fn generate_token_for_existing_applicant(
        &self,
        applicant_id: &str,
        level_name: &str,
    ) -> Result<String, SumsubError> {
        let path = format!("/resources/applicants/{}/accessTokens?levelName={}", applicant_id, level_name);
        let response = self.send_request(Method::POST, &path, None::<()>).await?;

        #[derive(Deserialize)]
        struct TokenResponse {
            token: String,
        }

        let token_response: TokenResponse = self.handle_response_and_deserialize(response).await?;
        Ok(token_response.token)
    }

    /// Retrieves similar applicants by text and face.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#retrieving-similar-applicantsduplicates)
    pub async fn get_similar_applicants_by_text_and_face(
        &self,
        applicant_id: &str,
    ) -> Result<SimilarByTextAndFaceResult, SumsubError> {
        let path = format!("/resources/applicants/{}/similar/byTextAndFace", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Retrieves applicant events/logs.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#retrieving-applicant-eventslogs)
    pub async fn get_applicant_events(
        &self,
        applicant_id: &str,
    ) -> Result<Vec<ApplicantEvent>, SumsubError> {
        let path = format!("/resources/applicants/{}/events", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Sends a verification email to the applicant.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#sending-verification-emails)
    pub async fn send_verification_email<'a>(
        &self,
        applicant_id: &str,
        request: SendVerificationMessageRequest<'a>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/info/email/send", applicant_id);
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Retrieves the liveness video.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#retrieving-liveness-resultsvideos)
    pub async fn get_liveness_video(
        &self,
        applicant_id: &str,
    ) -> Result<Vec<u8>, SumsubError> {
        let path = format!("/resources/applicants/{}/info/facemap/video", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(SumsubError::ApiError { status, message });
        }
        Ok(response.bytes().await?.to_vec())
    }

    /// Retrieves a PDF report of the verification.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#generating-pdf-reports)
    pub async fn get_verification_pdf_report(
        &self,
        applicant_id: &str,
    ) -> Result<Vec<u8>, SumsubError> {
        let path = format!("/resources/applicants/{}/requiredIdDocsStatus.pdf", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(SumsubError::ApiError { status, message });
        }
        Ok(response.bytes().await?.to_vec())
    }

    /// Changes applicant data in the `info` field.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#changing-applicant-data)
    pub async fn change_applicant_data(
        &self,
        applicant_id: &str,
        info: crate::models::Info,
    ) -> Result<crate::models::Applicant, SumsubError> {
        let path = format!("/resources/applicants/{}/info", applicant_id);
        let response = self.send_request(Method::PATCH, &path, Some(info)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Retrieves the list of available verification levels.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#retrieving-available-levels)
    pub async fn get_available_levels(&self) -> Result<Vec<AvailableLevel>, SumsubError> {
        let path = "/resources/sdkIntegrations/levels";
        let response = self.send_request(Method::GET, &path, None::<()>).await?;

        #[derive(Deserialize, Debug)]
        struct LevelsResponse {
            levels: Vec<AvailableLevel>,
        }
        let levels_response: LevelsResponse = self.handle_response_and_deserialize(response).await?;
        Ok(levels_response.levels)
    }

    /// Sends a verification SMS to the applicant's phone.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#sending-verification-sms)
    pub async fn send_verification_phone_sms<'a>(
        &self,
        applicant_id: &str,
        request: SendVerificationMessageRequest<'a>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/info/phone/send", applicant_id);
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Retrieves a ZIP archive report of the verification.
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#generating-pdf-reports)
    pub async fn get_verification_zip_report(
        &self,
        applicant_id: &str,
    ) -> Result<Vec<u8>, SumsubError> {
        let path = format!("/resources/applicants/{}/requiredIdDocsStatus.zip", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(SumsubError::ApiError { status, message });
        }
        Ok(response.bytes().await?.to_vec())
    }
}
