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
use crate::misc::{ApiHealthStatus, AuditTrailEvent, NewApplicantAccessTokenResponse, SendVerificationMessageRequest, AvailableLevel};
use crate::actions::{ApplicantAction, CreateApplicantActionRequest, GetApplicantActionsResponse, Questionnaire, RequestActionCheckResponse};
use crate::kyb::{CompanyInfo, GetAdditionalCompanyCheckDataResponse, LinkBeneficiaryRequest};
use crate::transactions::{BulkTransactionImportRequest, BulkTransactionImportResponse, DeleteTransactionResponse, SubmitTransactionRequest, SubmitTransactionResponse};
use crate::travel_rule::{ConfirmWalletOwnershipRequest, ImportWalletAddressesRequest, ImportWalletAddressesResponse, InitiateSdkRequest, InitiateSdkResponse, OwnershipStatus, PatchTransactionRequest, SetTransactionBlockRequest};
use crate::applicants::*;
use crate::checks::*;
use serde::Deserialize;
use urlencoding;


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
    base_url: String,
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
            base_url: BASE_URL.to_string(),
        }
    }

    /// Creates a new `Client` with a custom base URL for testing.
    pub fn new_with_base_url(app_token: String, secret_key: String, base_url: String) -> Self {
        Self {
            app_token,
            secret_key,
            http_client: reqwest::Client::new(),
            base_url,
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

        let url = format!("{}{}", self.base_url, path);
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

    /// Retrieves the latest PoA check result for an applicant.
    pub async fn get_latest_poa_check_result(
        &self,
        applicant_id: &str,
    ) -> Result<PoaCheckResult, SumsubError> {
        self.get_latest_check_result(applicant_id, CheckType::Poa)
            .await
    }

    /// Retrieves the latest bank card check result for an applicant.
    pub async fn get_latest_bank_card_check_result(
        &self,
        applicant_id: &str,
    ) -> Result<BankCardCheckResult, SumsubError> {
        self.get_latest_check_result(applicant_id, CheckType::BankCard)
            .await
    }

    /// Retrieves the latest email confirmation check result for an applicant.
    pub async fn get_latest_email_confirmation_check_result(
        &self,
        applicant_id: &str,
    ) -> Result<EmailConfirmationCheckResult, SumsubError> {
        self.get_latest_check_result(applicant_id, CheckType::EmailConfirmation)
            .await
    }

    /// Retrieves the latest phone confirmation check result for an applicant.
    pub async fn get_latest_phone_confirmation_check_result(
        &self,
        applicant_id: &str,
    ) -> Result<PhoneConfirmationCheckResult, SumsubError> {
        self.get_latest_check_result(applicant_id, CheckType::PhoneConfirmation)
            .await
    }

    /// Retrieves the latest IP check result for an applicant.
    pub async fn get_latest_ip_check_result(
        &self,
        applicant_id: &str,
    ) -> Result<IpCheckResult, SumsubError> {
        self.get_latest_check_result(applicant_id, CheckType::IpCheck)
            .await
    }

    /// Retrieves the latest NFC check result for an applicant.
    pub async fn get_latest_nfc_check_result(
        &self,
        applicant_id: &str,
    ) -> Result<NfcCheckResult, SumsubError> {
        self.get_latest_check_result(applicant_id, CheckType::Nfc)
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

        let url = format!("{}{}", self.base_url, path);
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

        let url = format!("{}{}", self.base_url, path);
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

    /// Generates an external WebSDK link.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#generate-external-websdk-link)
    pub async fn generate_external_websdk_link(
        &self,
        level_name: &str,
        external_user_id: Option<&str>,
        ttl_in_secs: Option<u64>,
    ) -> Result<GenerateWebsdkLinkResponse, SumsubError> {
        let path = "/resources/accessTokens/-/websdkLink";
        let request = GenerateWebsdkLinkRequest {
            level_name,
            external_user_id,
            ttl_in_secs,
        };
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_response_and_deserialize(response).await
    }

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

    /// Adds a verification document to an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#add-verification-documents)
    pub async fn add_verification_document(
        &self,
        applicant_id: &str,
        metadata: crate::applicants::AddDocumentMetadata<'_>,
        content: Vec<u8>,
        file_name: &str,
        mime_type: &str,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/docsets/-", applicant_id);

        let metadata_str = serde_json::to_string(&metadata)?;

        let part = reqwest::multipart::Part::bytes(content)
            .file_name(file_name.to_string())
            .mime_str(mime_type)
            .map_err(|e| SumsubError::MimeError(e.to_string()))?;

        let form = reqwest::multipart::Form::new()
            .part("metadata", reqwest::multipart::Part::text(metadata_str))
            .part("content", part);

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = sign_request(
            &self.secret_key,
            ts,
            "POST",
            &path,
            &None,
        );

        let url = format!("{}{}", self.base_url, &path);
        let response = self
            .http_client
            .post(&url)
            .header("X-App-Token", &self.app_token)
            .header("X-App-Access-Sig", signature)
            .header("X-App-Access-Ts", ts.to_string())
            .multipart(form)
            .send()
            .await?;

        self.handle_empty_response(response).await
    }

    /// Copies an applicant profile.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#copy-applicant-profile)
    pub async fn copy_applicant_profile(
        &self,
        applicant_id: &str,
    ) -> Result<crate::models::Applicant, SumsubError> {
        let path = format!("/resources/applicants/{}/duplicate", applicant_id);
        let response = self.send_request(Method::POST, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Simulates a review response in the Sandbox environment.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#simulate-review-response-in-sandbox)
    pub async fn simulate_review_response(
        &self,
        applicant_id: &str,
        request: crate::applicants::SimulateReviewRequest<'_>,
    ) -> Result<(), SumsubError> {
        let path = format!(
            "/resources/applicants/{}/sandbox/status/testCompleted",
            applicant_id
        );
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Runs an AML check for an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#run-aml-check)
    pub async fn run_aml_check(&self, applicant_id: &str) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/aml", applicant_id);
        let response = self.send_request(Method::POST, &path, None::<()>).await?;
        self.handle_empty_response(response).await
    }

    /// Gets AML case data for an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-aml-case-data)
    pub async fn get_aml_case_data(
        &self,
        applicant_id: &str,
    ) -> Result<crate::applicants::AmlData, SumsubError> {
        let path = format!("/resources/applicants/{}/aml", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Updates the review status of an AML hit.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#update-aml-hit-review)
    pub async fn update_aml_hit_review(
        &self,
        applicant_id: &str,
        hit_id: &str,
        request: crate::applicants::UpdateAmlHitReviewRequest<'_>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/aml/hits/{}", applicant_id, hit_id);
        let response = self.send_request(Method::PATCH, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Marks an image as inactive.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#marking-image-as-inactive)
    pub async fn mark_image_as_inactive(
        &self,
        applicant_id: &str,
        image_id: &str,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/images/{}", applicant_id, image_id);
        let response = self.send_request(Method::DELETE, &path, None::<()>).await?;
        self.handle_empty_response(response).await
    }

    /// Deactivates an applicant profile.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#deactivate-applicant-profile)
    pub async fn deactivate_applicant_profile(
        &self,
        applicant_id: &str,
        moderation_comment: Option<&str>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/deactivated", applicant_id);
        let request = crate::applicants::DeactivateApplicantRequest {
            review: crate::applicants::DeactivateApplicantReview {
                moderation_comment,
            },
        };
        let response = self.send_request(Method::PATCH, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Adds tags to an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#add-custom-applicant-tags)
    pub async fn add_applicant_tags<'a>(
        &self,
        applicant_id: &str,
        tags: Vec<&'a str>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/tags", applicant_id);
        let response = self.send_request(Method::POST, &path, Some(tags)).await?;
        self.handle_empty_response(response).await
    }

    /// Adds and overwrites tags for an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#adding-overwriting-custom-applicant-tags)
    pub async fn add_and_overwrite_applicant_tags<'a>(
        &self,
        applicant_id: &str,
        tags: Vec<&'a str>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/tags/-/overwrite", applicant_id);
        let response = self.send_request(Method::POST, &path, Some(tags)).await?;
        self.handle_empty_response(response).await
    }

    /// Removes tags from an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#remove-custom-applicant-tags)
    pub async fn remove_applicant_tags<'a>(
        &self,
        applicant_id: &str,
        tags: Vec<&'a str>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/tags", applicant_id);
        let response = self.send_request(Method::DELETE, &path, Some(tags)).await?;
        self.handle_empty_response(response).await
    }

    /// Adds accepted consents for an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#add-accepted-applicant-consents)
    pub async fn add_applicant_consents<'a>(
        &self,
        applicant_id: &str,
        consents: Vec<&'a str>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/consents", applicant_id);
        let request = crate::applicants::AddConsentsRequest { accepted: consents };
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Gets the applicant-facing consents for a given level.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-applicant-facing-consents)
    pub async fn get_applicant_facing_consents(
        &self,
        level_name: &str,
    ) -> Result<crate::applicants::ApplicantFacingConsentsResponse, SumsubError> {
        let path = format!("/resources/sdkIntegrations/levels/{}/consents", level_name);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Gets notes for an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-applicant-notes)
    pub async fn get_applicant_notes(
        &self,
        applicant_id: &str,
    ) -> Result<Vec<crate::applicants::Note>, SumsubError> {
        let path = format!("/resources/applicants/{}/notes", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Adds a note to an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#add-applicant-note)
    pub async fn add_applicant_note<'a>(
        &self,
        applicant_id: &str,
        note: &'a str,
    ) -> Result<crate::applicants::Note, SumsubError> {
        let path = format!("/resources/applicants/{}/notes", applicant_id);
        let request = crate::applicants::AddNoteRequest { note };
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Edits an applicant note.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#edit-applicant-note)
    pub async fn edit_applicant_note<'a>(
        &self,
        applicant_id: &str,
        note_id: &str,
        note: &'a str,
    ) -> Result<crate::applicants::Note, SumsubError> {
        let path = format!("/resources/applicants/{}/notes/{}", applicant_id, note_id);
        let request = crate::applicants::EditNoteRequest { note };
        let response = self.send_request(Method::PATCH, &path, Some(request)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Removes an applicant note.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#remove-applicant-note)
    pub async fn remove_applicant_note(
        &self,
        applicant_id: &str,
        note_id: &str,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/notes/{}", applicant_id, note_id);
        let response = self.send_request(Method::DELETE, &path, None::<()>).await?;
        self.handle_empty_response(response).await
    }

    /// Adds an attachment to an applicant note.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#add-attachment-to-applicant-note)
    pub async fn add_note_attachment(
        &self,
        applicant_id: &str,
        note_id: &str,
        content: Vec<u8>,
        file_name: &str,
        mime_type: &str,
    ) -> Result<crate::applicants::Note, SumsubError> {
        let path = format!("/resources/applicants/{}/notes/{}/attachments", applicant_id, note_id);

        let part = reqwest::multipart::Part::bytes(content)
            .file_name(file_name.to_string())
            .mime_str(mime_type)
            .map_err(|e| SumsubError::MimeError(e.to_string()))?;

        let form = reqwest::multipart::Form::new().part("content", part);

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = sign_request(
            &self.secret_key,
            ts,
            "POST",
            &path,
            &None,
        );

        let url = format!("{}{}", self.base_url, &path);
        let response = self
            .http_client
            .post(&url)
            .header("X-App-Token", &self.app_token)
            .header("X-App-Access-Sig", signature)
            .header("X-App-Access-Ts", ts.to_string())
            .multipart(form)
            .send()
            .await?;

        self.handle_response_and_deserialize(response).await
    }

    /// Downloads an attachment from a note.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#download-note-attachment)
    pub async fn download_note_attachment(
        &self,
        applicant_id: &str,
        note_id: &str,
        attachment_id: &str,
    ) -> Result<Vec<u8>, SumsubError> {
        let path = format!(
            "/resources/applicants/{}/notes/{}/attachments/{}",
            applicant_id, note_id, attachment_id
        );
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(SumsubError::ApiError { status, message });
        }
        Ok(response.bytes().await?.to_vec())
    }

    /// Removes an attachment from a note.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#remove-note-attachment)
    pub async fn remove_note_attachment(
        &self,
        applicant_id: &str,
        note_id: &str,
        attachment_id: &str,
    ) -> Result<(), SumsubError> {
        let path = format!(
            "/resources/applicants/{}/notes/{}/attachments/{}",
            applicant_id, note_id, attachment_id
        );
        let response = self.send_request(Method::DELETE, &path, None::<()>).await?;
        self.handle_empty_response(response).await
    }

    /// Gets applicant data by external user ID.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-applicant-data-externaluserid)
    pub async fn get_applicant_data_by_external_user_id(
        &self,
        external_user_id: &str,
    ) -> Result<crate::models::Applicant, SumsubError> {
        let path = format!("/resources/applicants/-;externalUserId={}/one", external_user_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Gets the status of verification steps for an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-status-of-verification-steps)
    pub async fn get_verification_steps_status(
        &self,
        applicant_id: &str,
    ) -> Result<std::collections::HashMap<String, crate::applicants::VerificationStepStatus>, SumsubError> {
        let path = format!("/resources/applicants/{}/requiredIdDocsStatus", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Gets the review history for an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-applicant-review-history)
    pub async fn get_applicant_review_history(
        &self,
        applicant_id: &str,
    ) -> Result<Vec<crate::applicants::ReviewHistoryRecord>, SumsubError> {
        let path = format!("/resources/applicants/{}/review/history", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Gets a document image.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-document-images)
    pub async fn get_document_image(
        &self,
        applicant_id: &str,
        inspection_id: &str,
        image_id: &str,
    ) -> Result<Vec<u8>, SumsubError> {
        let path = format!("/resources/applicants/{}/images/{}/{}", applicant_id, inspection_id, image_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(SumsubError::ApiError { status, message });
        }
        Ok(response.bytes().await?.to_vec())
    }

    /// Gets information about document images for an applicant.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-information-about-document-images)
    pub async fn get_document_images_info(
        &self,
        applicant_id: &str,
    ) -> Result<Vec<crate::applicants::ImageInfo>, SumsubError> {
        let path = format!("/resources/applicants/{}/info/images", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Adds an image to an applicant action.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#add-images-to-applicant-actions)
    pub async fn add_image_to_action(
        &self,
        action_id: &str,
        metadata: Option<crate::actions::AddActionImageMetadata<'_>>,
        content: Vec<u8>,
        file_name: &str,
        mime_type: &str,
    ) -> Result<Vec<crate::actions::ActionImage>, SumsubError> {
        let path = format!("/resources/applicantActions/{}/images", action_id);

        let part = reqwest::multipart::Part::bytes(content)
            .file_name(file_name.to_string())
            .mime_str(mime_type)
            .map_err(|e| SumsubError::MimeError(e.to_string()))?;

        let mut form = reqwest::multipart::Form::new().part("content", part);
        if let Some(metadata) = metadata {
            let metadata_str = serde_json::to_string(&metadata)?;
            form = form.part("metadata", reqwest::multipart::Part::text(metadata_str));
        }

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = sign_request(
            &self.secret_key,
            ts,
            "POST",
            &path,
            &None,
        );

        let url = format!("{}{}", self.base_url, &path);
        let response = self
            .http_client
            .post(&url)
            .header("X-App-Token", &self.app_token)
            .header("X-App-Access-Sig", signature)
            .header("X-App-Access-Ts", ts.to_string())
            .multipart(form)
            .send()
            .await?;

        self.handle_response_and_deserialize(response).await
    }

    /// Gets an image from an applicant action.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-images-from-applicant-actions)
    pub async fn get_image_from_action(
        &self,
        action_id: &str,
        image_id: &str,
    ) -> Result<Vec<u8>, SumsubError> {
        let path = format!("/resources/applicantActions/{}/images/{}", action_id, image_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(SumsubError::ApiError { status, message });
        }
        Ok(response.bytes().await?.to_vec())
    }

    /// Gets OCR fields from company documents.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-ocr-fields-from-company-documents)
    pub async fn get_ocr_fields_from_company_documents(
        &self,
        applicant_id: &str,
    ) -> Result<std::collections::HashMap<String, String>, SumsubError> {
        let path = format!("/resources/applicants/{}/info/companyInfo/ocr", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Submits applicant data for Non-Doc Verification.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#submit-applicant-data)
    pub async fn submit_non_doc_data(
        &self,
        applicant_id: &str,
        data: serde_json::Value,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/info/nondoc", applicant_id);
        let response = self.send_request(Method::POST, &path, Some(data)).await?;
        self.handle_empty_response(response).await
    }

    /// Confirms applicant data for Non-Doc Verification.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#confirm-applicant-data)
    pub async fn confirm_non_doc_data<'a>(
        &self,
        applicant_id: &str,
        consent: &'a str,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/info/nondoc/confirm", applicant_id);
        let request = crate::non_doc::ConfirmNonDocDataRequest { consent };
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Gets applicant data from Non-Doc Verification.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-non-doc-applicant-data)
    pub async fn get_non_doc_data(
        &self,
        applicant_id: &str,
    ) -> Result<serde_json::Value, SumsubError> {
        let path = format!("/resources/applicants/{}/info/nondoc", applicant_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Approves or rejects a transaction.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#approve-and-reject-transaction)
    pub async fn review_transaction<'a>(
        &self,
        txn_id: &str,
        action: crate::transactions::TransactionReviewAction,
        moderation_comment: Option<&'a str>,
    ) -> Result<crate::transactions::SubmitTransactionResponse, SumsubError> {
        let path = format!("/resources/kyt/txns/{}/review/{}", txn_id, action.to_string());
        let request = crate::transactions::ReviewTransactionRequest {
            review: crate::transactions::ReviewTransactionDetails {
                moderation_comment,
            },
        };
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Rescores a transaction.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#re-score-transaction)
    pub async fn rescore_transaction(
        &self,
        txn_id: &str,
    ) -> Result<crate::transactions::SubmitTransactionResponse, SumsubError> {
        let path = format!("/resources/kyt/txns/{}/rescore", txn_id);
        let response = self.send_request(Method::POST, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Changes transaction properties.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#changing-transaction-custom-properties)
    pub async fn change_transaction_properties(
        &self,
        txn_id: &str,
        properties: serde_json::Value,
    ) -> Result<crate::transactions::SubmitTransactionResponse, SumsubError> {
        let path = format!("/resources/kyt/txns/{}/info", txn_id);
        let response = self.send_request(Method::PATCH, &path, Some(properties)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Finds specific transactions using an expression.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#find-specific-transactions)
    pub async fn find_transactions(
        &self,
        expression: &str,
    ) -> Result<crate::transactions::FindTransactionsResponse, SumsubError> {
        let encoded_expression = urlencoding::encode(expression);
        let path = format!("/resources/kyt/txns/search?expression={}", encoded_expression);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Gets the list of available currencies for transaction monitoring.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-available-currencies)
    pub async fn get_available_currencies(
        &self,
    ) -> Result<crate::transactions::AvailableCurrenciesResponse, SumsubError> {
        let path = "/resources/kyt/misc/availableCurrencies";
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Adds tags to a transaction.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#add-txn-tags)
    pub async fn add_transaction_tags<'a>(
        &self,
        txn_id: &str,
        tags: Vec<&'a str>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/kyt/txns/{}/tags", txn_id);
        let request = crate::transactions::AddTransactionTagsRequest { tags };
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Gets tags for a transaction.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-txn-tags)
    pub async fn get_transaction_tags(
        &self,
        txn_id: &str,
    ) -> Result<crate::transactions::GetTransactionTagsResponse, SumsubError> {
        let path = format!("/resources/kyt/txns/{}/tags", txn_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Removes tags from a transaction.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#remove-txn-tags)
    pub async fn remove_transaction_tags<'a>(
        &self,
        txn_id: &str,
        tags: Vec<&'a str>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/kyt/txns/{}/tags", txn_id);
        let request = crate::transactions::RemoveTransactionTagsRequest { tags };
        let response = self.send_request(Method::DELETE, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Adds a note to a transaction.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#add-txn-notes)
    pub async fn add_transaction_note<'a>(
        &self,
        txn_id: &str,
        note: &'a str,
    ) -> Result<crate::transactions::TransactionNote, SumsubError> {
        let path = format!("/resources/kyt/txns/{}/notes", txn_id);
        let request = crate::transactions::AddTransactionNoteRequest { note };
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Gets notes for a transaction.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-txn-notes)
    pub async fn get_transaction_notes(
        &self,
        txn_id: &str,
    ) -> Result<Vec<crate::transactions::TransactionNote>, SumsubError> {
        let path = format!("/resources/kyt/txns/{}/notes", txn_id);
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Edits a transaction note.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#edit-txn-notes)
    pub async fn edit_transaction_note<'a>(
        &self,
        txn_id: &str,
        note_id: &str,
        note: &'a str,
    ) -> Result<crate::transactions::TransactionNote, SumsubError> {
        let path = format!("/resources/kyt/txns/{}/notes/{}", txn_id, note_id);
        let request = crate::transactions::EditTransactionNoteRequest { note };
        let response = self.send_request(Method::PATCH, &path, Some(request)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Removes a transaction note.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#remove-txn-notes)
    pub async fn remove_transaction_note(
        &self,
        txn_id: &str,
        note_id: &str,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/kyt/txns/{}/notes/{}", txn_id, note_id);
        let response = self.send_request(Method::DELETE, &path, None::<()>).await?;
        self.handle_empty_response(response).await
    }

    /// Adds a payment method.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#add-payment-method)
    pub async fn add_payment_method(
        &self,
        payment_method: crate::transactions::PaymentMethod,
    ) -> Result<crate::transactions::PaymentMethod, SumsubError> {
        let path = "/resources/kyt/misc/paymentMethods";
        let response = self.send_request(Method::POST, &path, Some(payment_method)).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Updates a wallet address.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#update-wallet-address)
    pub async fn update_wallet_address(
        &self,
        address: &str,
        request: crate::travel_rule::UpdateWalletAddressRequest,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/kyt/txns/info/address/{}", address);
        let response = self.send_request(Method::PATCH, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Gets the list of available VASPs.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#get-available-vasps)
    pub async fn get_available_vasps(&self) -> Result<crate::travel_rule::VaspsResponse, SumsubError> {
        let path = "/resources/kyt/vasps";
        let response = self.send_request(Method::GET, &path, None::<()>).await?;
        self.handle_response_and_deserialize(response).await
    }

    /// Generates a Device Intelligence access token.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#generate-access-token-device-intelligence)
    pub async fn generate_device_intelligence_token(
        &self,
        lang: Option<&str>,
    ) -> Result<String, SumsubError> {
        let path = "/resources/accessTokens?type=device";
        let request_body = if let Some(lang) = lang {
            Some(serde_json::json!({ "lang": lang }))
        } else {
            None
        };
        let response = self.send_request(Method::POST, &path, request_body).await?;

        #[derive(Deserialize)]
        struct TokenResponse {
            token: String,
        }

        let token_response: TokenResponse = self.handle_response_and_deserialize(response).await?;
        Ok(token_response.token)
    }

    /// Sends an applicant platform event with captured device information.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#send-applicant-platform-event-with-captured-device)
    pub async fn send_platform_event(
        &self,
        applicant_id: &str,
        event: crate::device_intelligence::PlatformEvent<'_>,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/applicants/{}/platformEvents", applicant_id);
        let response = self.send_request(Method::POST, &path, Some(event)).await?;
        self.handle_empty_response(response).await
    }

    /// Sends financial transaction data with captured device information.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#send-financial-transaction-with-captured-device)
    pub async fn send_financial_transaction_with_device(
        &self,
        txn_id: &str,
        fingerprint: &str,
    ) -> Result<(), SumsubError> {
        let path = format!("/resources/kyt/txns/{}/data/applicant/device", txn_id);
        let request = crate::device_intelligence::DeviceFingerprint { fingerprint };
        let response = self.send_request(Method::POST, &path, Some(request)).await?;
        self.handle_empty_response(response).await
    }

    /// Imports an applicant profile from a zip archive.
    ///
    /// [Sumsub API reference](https://developers.sumsub.com/api-reference/#import-applicant-profile-from-archive)
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the zip archive.
    /// * `file_name` - The name of the file.
    pub async fn import_applicant_profile_from_archive(
        &self,
        content: Vec<u8>,
        file_name: &str,
    ) -> Result<(), SumsubError> {
        let path = "/resources/applicants/-/ingest";

        let part = reqwest::multipart::Part::bytes(content)
            .file_name(file_name.to_string())
            .mime_str("application/zip")
            .map_err(|e| SumsubError::MimeError(e.to_string()))?;

        let form = reqwest::multipart::Form::new().part("content", part);

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = sign_request(
            &self.secret_key,
            ts,
            "POST",
            path,
            &None,
        );

        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http_client
            .post(&url)
            .header("X-App-Token", &self.app_token)
            .header("X-App-Access-Sig", signature)
            .header("X-App-Access-Ts", ts.to_string())
            .multipart(form)
            .send()
            .await?;

        self.handle_empty_response(response).await
    }
}
