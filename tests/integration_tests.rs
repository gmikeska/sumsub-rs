use sumsub_api::client::Client;
use sumsub_api::error::SumsubError;
use sumsub_api::models::{CreateApplicantRequest, FixedInfo};
use sumsub_api::applicants::AddDocumentMetadata;
use sumsub_api::webhooks;
use sumsub_api::actions::AddActionImageMetadata;
use sumsub_api::transactions::TransactionReviewAction;
use sumsub_api::travel_rule::UpdateWalletAddressRequest;
use sumsub_api::device_intelligence::{PlatformEvent, DeviceFingerprint};

use mockito;
use uuid::Uuid;
use hex;
use serde_json::json;

// Helper function to generate HMAC-SHA1 signature for testing
fn generate_webhook_signature(secret_key: &str, payload: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha1::Sha1;

    let mut mac = Hmac::<Sha1>::new_from_slice(secret_key.as_bytes())
        .expect("Failed to create HMAC-SHA1 instance");
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    hex::encode(code_bytes)
}

#[tokio::test]
async fn test_get_api_health_status() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let mock = server.mock("GET", "/resources/status/api")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "ok"}"#)
        .create_async().await;

    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let result = client.get_api_health_status().await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let health_status = result.unwrap();
    assert_eq!(health_status.status, "ok");
}

#[tokio::test]
async fn test_create_and_get_applicant() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let external_user_id = Uuid::new_v4().to_string();
    let applicant_id = Uuid::new_v4().to_string();
    let inspection_id = Uuid::new_v4().to_string();
    let level_name = "basic-kyc";

    let create_applicant_request = CreateApplicantRequest {
        external_user_id: external_user_id.clone(),
        ..Default::default()
    };

    let response_body = serde_json::json!({
        "id": applicant_id,
        "createdAt": "2023-10-26T10:00:00Z",
        "clientId": "some_client_id",
        "inspectionId": inspection_id,
        "externalUserId": external_user_id,
        "review": {
            "reviewStatus": "pending"
        },
        "type": "individual",
        "applicantPlatform": "api"
    });

    let mock_create = server.mock("POST", "/resources/applicants?levelName=basic-kyc")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async().await;

    let created_applicant = client.create_applicant(create_applicant_request, level_name).await.unwrap();
    assert_eq!(created_applicant.id, applicant_id);

    mock_create.assert_async().await;

    let mock_get = server.mock("GET", &format!("/resources/applicants/{}/one", applicant_id)[..])
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async().await;

    let fetched_applicant = client.get_applicant_data(&applicant_id).await.unwrap();
    assert_eq!(fetched_applicant.id, applicant_id);

    mock_get.assert_async().await;
}

#[tokio::test]
async fn test_get_applicant_data_not_found() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let applicant_id = "non_existent_id";

    let mock = server.mock("GET", &format!("/resources/applicants/{}/one", applicant_id)[..])
        .with_status(404)
        .create_async().await;

    let result = client.get_applicant_data(applicant_id).await;

    mock.assert_async().await;

    assert!(result.is_err());
    match result.err().unwrap() {
        SumsubError::ApiError { status, .. } => assert_eq!(status, 404),
        _ => panic!("Expected ApiError"),
    }
}

#[tokio::test]
async fn test_add_verification_document() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let applicant_id = "some_applicant_id";

    let mock = server.mock("POST", &format!("/resources/applicants/{}/docsets/-", applicant_id)[..])
        .with_status(201)
        .match_header("content-type", mockito::Matcher::Regex(r"multipart/form-data; boundary=.+".to_string()))
        .create_async().await;

    let metadata = AddDocumentMetadata {
        id_doc_type: "PASSPORT",
        country: "USA",
        first_name: Some("John"),
        middle_name: None,
        last_name: Some("Doe"),
        dob: Some("1990-01-01"),
        place_of_birth: None,
        issued_date: None,
        valid_until: None,
        number: None,
        sub_type: None,
        id_doc_sub_type: None,
    };
    let content = vec![1, 2, 3];
    let file_name = "passport.jpg";
    let mime_type = "image/jpeg";

    let result = client.add_verification_document(applicant_id, metadata, content, file_name, mime_type).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[test]
fn test_webhook_signature_verification() {
    let secret_key = "my_secret_key";
    let payload = r#"{"type": "applicantReviewed", "applicantId": "..."}"#;

    let signature = generate_webhook_signature(secret_key, payload);

    let result = webhooks::verify_signature(secret_key, payload.as_bytes(), &signature);
    assert!(result.is_ok());

    let invalid_signature = "invalid_signature";
    let result = webhooks::verify_signature(secret_key, payload.as_bytes(), invalid_signature);
    assert!(result.is_err());

    let invalid_payload = r#"{"type": "applicantReviewed", "applicantId": "different"}"#;
    let result = webhooks::verify_signature(secret_key, invalid_payload.as_bytes(), &signature);
    assert!(result.is_err());
}

#[test]
fn test_webhook_payload_deserialization() {
    let payload = r#"
    {
        "type": "applicantReviewed",
        "applicantId": "some_applicant_id",
        "inspectionId": "some_inspection_id",
        "correlationId": "some_correlation_id",
        "levelName": "basic-kyc",
        "externalUserId": "some_external_id",
        "review": {
            "reviewId": "some_review_id",
            "attemptId": "some_attempt_id",
            "attemptCnt": 1,
            "elapsedSincePendingMs": 1000,
            "createDate": "2023-10-26T10:00:00Z",
            "reviewStatus": "completed",
            "reviewResult": {
                "reviewAnswer": "GREEN"
            }
        },
        "createdAt": "2023-10-26T10:00:00Z",
        "applicantType": "individual"
    }
    "#;

    let result: Result<webhooks::WebhookPayload, _> = serde_json::from_str(payload);
    assert!(result.is_ok());

    match result.unwrap() {
        webhooks::WebhookPayload::ApplicantReviewed(payload) => {
            assert_eq!(payload.applicant_id, "some_applicant_id");
            assert_eq!(payload.review.review_result.unwrap().review_answer, "GREEN");
        }
        _ => panic!("Expected ApplicantReviewed payload"),
    }
}

#[tokio::test]
async fn test_move_applicant_to_level() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let applicant_id = "some_applicant_id";
    let level_name = "new-level";

    let mock = server.mock("POST", &format!("/resources/applicants/{}/moveToLevel?levelName={}", applicant_id, level_name)[..])
        .with_status(200)
        .create_async().await;

    let result = client.move_applicant_to_level(applicant_id, level_name).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_applicant_fixed_info() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let applicant_id = "some_applicant_id";
    let fixed_info = FixedInfo {
        first_name: Some("Jane".to_string()),
        ..Default::default()
    };

    let mock = server.mock("PATCH", &format!("/resources/applicants/{}/fixedInfo", applicant_id)[..])
        .with_status(200)
        .create_async().await;

    let result = client.update_applicant_fixed_info(applicant_id, fixed_info).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_applicant_status() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let applicant_id = "some_applicant_id";
    let response_body = serde_json::json!({
        "createDate": "2023-10-26T10:00:00Z",
        "reviewStatus": "completed"
    });

    let mock = server.mock("GET", &format!("/resources/applicants/{}/status", applicant_id)[..])
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async().await;

    let result = client.get_applicant_status(applicant_id).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.review_status, "completed");
}

#[tokio::test]
async fn test_add_image_to_action() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let action_id = "some_action_id";
    let image_id = "some_image_id";

    let response_body = serde_json::json!([{
        "imageId": image_id,
        "addedAt": "2023-10-26T10:00:00Z",
        "idDocType": "ID_CARD",
        "idDocSubType": "FRONT_SIDE"
    }]);

    let mock = server.mock("POST", &format!("/resources/applicantActions/{}/images", action_id)[..])
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async().await;

    let metadata = AddActionImageMetadata {
        country: Some("USA"),
        id_doc_type: Some("ID_CARD"),
        ..Default::default()
    };
    let content = vec![1, 2, 3];
    let file_name = "id_card.jpg";
    let mime_type = "image/jpeg";

    let result = client.add_image_to_action(action_id, Some(metadata), content, file_name, mime_type).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let images = result.unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].image_id, image_id);
}

#[tokio::test]
async fn test_get_image_from_action() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let action_id = "some_action_id";
    let image_id = "some_image_id";
    let image_content = vec![1, 2, 3, 4, 5];

    let mock = server.mock("GET", &format!("/resources/applicantActions/{}/images/{}", action_id, image_id)[..])
        .with_status(200)
        .with_header("content-type", "image/jpeg")
        .with_body(&image_content)
        .create_async().await;

    let result = client.get_image_from_action(action_id, image_id).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), image_content);
}

#[tokio::test]
async fn test_review_transaction() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let txn_id = "some_txn_id";
    let response_body = serde_json::json!({
        "id": "some_id",
        "createdAt": "2023-10-26T10:00:00Z",
        "clientId": "some_client_id",
        "applicantId": "some_applicant_id",
        "txnId": txn_id,
        "type": "withdrawal",
        "review": {
            "reviewId": "some_review_id",
            "attemptId": "some_attempt_id",
            "attemptCnt": 1,
            "levelName": "basic-kyt",
            "createDate": "2023-10-26T10:00:00Z",
            "reviewStatus": "completed",
            "reviewResult": {
                "reviewAnswer": "GREEN"
            }
        }
    });

    let mock = server.mock("POST", &format!("/resources/kyt/txns/{}/review/approve", txn_id)[..])
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async().await;

    let result = client.review_transaction(txn_id, TransactionReviewAction::Approve, Some("Looks good")).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let txn = result.unwrap();
    assert_eq!(txn.txn_id, txn_id);
}

#[tokio::test]
async fn test_rescore_transaction() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let txn_id = "some_txn_id";
    let response_body = serde_json::json!({
        "id": "some_id",
        "createdAt": "2023-10-26T10:00:00Z",
        "clientId": "some_client_id",
        "applicantId": "some_applicant_id",
        "txnId": txn_id,
        "type": "withdrawal",
        "review": {
            "reviewId": "some_review_id",
            "attemptId": "some_attempt_id",
            "attemptCnt": 1,
            "levelName": "basic-kyt",
            "createDate": "2023-10-26T10:00:00Z",
            "reviewStatus": "completed"
        }
    });

    let mock = server.mock("POST", &format!("/resources/kyt/txns/{}/rescore", txn_id)[..])
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async().await;

    let result = client.rescore_transaction(txn_id).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_available_currencies() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let response_body = serde_json::json!({
        "currencies": ["BTC", "ETH"]
    });

    let mock = server.mock("GET", "/resources/kyt/misc/availableCurrencies")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async().await;

    let result = client.get_available_currencies().await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let currencies = result.unwrap();
    assert_eq!(currencies.currencies, vec!["BTC", "ETH"]);
}

#[tokio::test]
async fn test_generate_device_intelligence_token() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let token = "some_device_token";
    let response_body = serde_json::json!({ "token": token });

    let mock = server.mock("POST", "/resources/accessTokens?type=device")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async().await;

    let result = client.generate_device_intelligence_token(Some("en")).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), token);
}

#[tokio::test]
async fn test_send_platform_event() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let applicant_id = "some_applicant_id";
    let event = PlatformEvent {
        event_type: "login",
        event_timestamp: "2023-10-26T10:00:00Z",
        correlation_id: "some_correlation_id",
        device: DeviceFingerprint {
            fingerprint: "some_fingerprint",
        },
    };

    let mock = server.mock("POST", &format!("/resources/applicants/{}/platformEvents", applicant_id)[..])
        .with_status(201)
        .create_async().await;

    let result = client.send_platform_event(applicant_id, event).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_financial_transaction_with_device() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let txn_id = "some_txn_id";
    let fingerprint = "some_fingerprint";

    let mock = server.mock("POST", &format!("/resources/kyt/txns/{}/data/applicant/device", txn_id)[..])
        .with_status(201)
        .create_async().await;

    let result = client.send_financial_transaction_with_device(txn_id, fingerprint).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_available_vasps() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let response_body = json!({
        "list": {
            "items": [
                {
                    "id": "some-id",
                    "name": "Some VASP",
                    "website": "https://example.com",
                    "logo": "https://example.com/logo.png",
                    "isTest": false
                }
            ]
        }
    });

    let mock = server.mock("GET", "/resources/kyt/vasps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async().await;

    let result = client.get_available_vasps().await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let vasps = result.unwrap();
    assert_eq!(vasps.list.items.len(), 1);
    assert_eq!(vasps.list.items[0].name, "Some VASP");
}

#[tokio::test]
async fn test_update_wallet_address() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let address = "some_address";
    let request = UpdateWalletAddressRequest {
        is_favorite: Some(true),
        props: None,
    };

    let mock = server.mock("PATCH", &format!("/resources/kyt/txns/info/address/{}", address)[..])
        .with_status(200)
        .create_async().await;

    let result = client.update_wallet_address(address, request).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_applicant_tags() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let applicant_id = "some_applicant_id";
    let tags = vec!["tag1", "tag2"];

    let mock = server.mock("POST", &format!("/resources/applicants/{}/tags", applicant_id)[..])
        .with_status(200)
        .create_async().await;

    let result = client.add_applicant_tags(applicant_id, tags).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_transaction_tags() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let txn_id = "some_txn_id";
    let tags = vec!["tag1", "tag2"];

    let mock = server.mock("POST", &format!("/resources/kyt/txns/{}/tags", txn_id)[..])
        .with_status(200)
        .create_async().await;

    let result = client.add_transaction_tags(txn_id, tags).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_applicant_note() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let client = Client::new_with_base_url("app_token".to_string(), "secret_key".to_string(), url);

    let applicant_id = "some_applicant_id";
    let note_text = "This is a test note.";
    let response_body = json!({
        "id": "some_note_id",
        "createdAt": "2023-10-26T10:00:00Z",
        "applicantId": applicant_id,
        "agent": {
            "clientId": "some_client_id",
            "email": "agent@example.com"
        },
        "note": note_text,
        "attachments": []
    });

    let mock = server.mock("POST", &format!("/resources/applicants/{}/notes", applicant_id)[..])
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async().await;

    let result = client.add_applicant_note(applicant_id, note_text).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let note = result.unwrap();
    assert_eq!(note.note, note_text);
}
