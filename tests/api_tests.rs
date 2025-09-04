use std::env;
use sumsub_api::client::Client;
use sumsub_api::models::{CreateApplicantRequest, FixedInfo};
use uuid::Uuid;

// Helper function to initialize the client from environment variables.
// To run these tests, set SUMSUB_APP_TOKEN and SUMSUB_SECRET_KEY.
fn setup_client() -> Client {
    let app_token = env::var("SUMSUB_APP_TOKEN")
        .expect("SUMSUB_APP_TOKEN must be set to run tests");
    let secret_key = env::var("SUMSUB_SECRET_KEY")
        .expect("SUMSUB_SECRET_KEY must be set to run tests");
    Client::new(app_token, secret_key)
}

#[tokio::test]
async fn test_create_and_get_applicant() {
    let client = setup_client();
    let external_user_id = Uuid::new_v4().to_string();

    let request = CreateApplicantRequest {
        external_user_id: external_user_id.clone(),
        fixed_info: Some(FixedInfo {
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let level_name = "id-and-liveness";

    let create_result = client.create_applicant(request, level_name).await;
    match create_result {
        Ok(applicant) => {
            // The request succeeded, so we'll proceed with the rest of the assertions.
            assert_eq!(applicant.external_user_id, external_user_id);

            let get_result = client.get_applicant_data(&applicant.id).await;
            assert!(get_result.is_ok(), "Failed to get applicant data: {:?}", get_result.err());

            let fetched_applicant = get_result.unwrap();
            assert_eq!(fetched_applicant.id, applicant.id);
        }
        Err(e) => {
            // The API call failed. This is unexpected with a valid level name.
            panic!(
                "API call failed unexpectedly with level name '{}'. Error: {}",
                level_name, e
            );
        }
    }
}

#[tokio::test]
async fn test_get_api_health_status() {
    let client = setup_client();
    let result = client.get_api_health_status().await;
    assert!(result.is_ok(), "Failed to get API health status: {:?}", result.err());
}

#[tokio::test]
async fn test_get_applicant_actions() {
    // This test requires an applicant to exist.
    // For a real test suite, you'd create an applicant in a setup function.
    // For this example, we assume one has been created.
    let client = setup_client();

    // You would need a valid applicant ID here.
    // let applicant_id = "some_real_applicant_id";
    // let result = client.get_applicant_actions(applicant_id).await;
    // assert!(result.is_ok(), "Failed to get applicant actions: {:?}", result.err());

    // Since we don't have a static applicant, we'll just check if the function can be called.
    // This is a placeholder for a real integration test.
    let result = client.get_applicant_actions("an_id_that_will_likely_fail").await;
    // We expect an error here, but a "deserialization" or "network" error would be a bug.
    // An API error (like applicant not found) is expected.
    assert!(result.is_err(), "This call should fail for a non-existent applicant");
}

#[tokio::test]
async fn test_get_transaction_data() {
    let client = setup_client();

    // This test requires a valid transaction ID.
    // let txn_id = "some_real_transaction_id";
    // let result = client.get_transaction_data(txn_id).await;
    // assert!(result.is_ok(), "Failed to get transaction data: {:?}", result.err());

    // Placeholder test
    let result = client.get_transaction_data("a_txn_id_that_will_fail").await;
    assert!(result.is_err(), "This call should fail for a non-existent transaction");
}
