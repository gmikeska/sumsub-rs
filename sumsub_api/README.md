# Sumsub API Rust Crate

[![Crates.io](https://img.shields.io/crates/v/sumsub_api.svg)](https://crates.io/crates/sumsub_api) <!-- Placeholder -->
[![Docs.rs](https://docs.rs/sumsub_api/badge.svg)](https://docs.rs/sumsub_api) <!-- Placeholder -->

An unofficial, asynchronous Rust client for the [Sumsub API](https://docs.sumsub.com/reference/about-sumsub-api).

This crate provides a convenient, type-safe interface for interacting with the Sumsub API, allowing you to perform actions such as creating applicants, managing verification checks, monitoring transactions, and more, all from within a Rust application.

## Features

*   Asynchronous API using `reqwest` and `tokio`.
*   Strongly-typed models for API requests and responses using `serde`.
*   Automatic request signing for Sumsub authentication.
*   Comprehensive coverage of major API endpoints.
*   Custom error type for easy error handling.

## Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
sumsub_api = "0.1.0" # Replace with the desired version from crates.io
```
*(Note: As this crate is not yet published, you may need to install it from a git source.)*

```toml
[dependencies]
sumsub_api = { git = "https://github.com/your-repo/sumsub-api-rs" } # Replace with actual URL
```

You will also need `tokio` for the async runtime.

## Usage

First, create a `Client` instance with your Sumsub App Token and Secret Key.

```rust
use sumsub_api::client::Client;

let app_token = "YOUR_SUMSUB_APP_TOKEN".to_string();
let secret_key = "YOUR_SUMSUB_SECRET_KEY".to_string();

let client = Client::new(app_token, secret_key);
```

Then, use the client to make API calls. All API methods are `async`.

### Example: Create and Fetch an Applicant

Here is a basic example of how to create an applicant and then retrieve their data:

```rust
use sumsub_api::client::Client;
use sumsub_api::models::CreateApplicantRequest;

#[tokio::main]
async fn main() {
    let app_token = std::env::var("SUMSUB_APP_TOKEN").expect("SUMSUB_APP_TOKEN not set");
    let secret_key = std::env::var("SUMSUB_SECRET_KEY").expect("SUMSUB_SECRET_KEY not set");

    let client = Client::new(app_token, secret_key);

    let request = CreateApplicantRequest {
        external_user_id: "some-unique-user-id-from-your-system".to_string(),
        ..Default::default()
    };

    let level_name = "basic-kyc-level";

    match client.create_applicant(request, level_name).await {
        Ok(applicant) => {
            println!("Successfully created applicant: {:#?}", applicant);

            // Now, fetch the applicant's data
            match client.get_applicant_data(&applicant.id).await {
                Ok(fetched_applicant) => {
                    println!("Successfully fetched applicant data: {:#?}", fetched_applicant);
                }
                Err(e) => {
                    eprintln!("Error fetching applicant data: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error creating applicant: {}", e);
        }
    }
}
```
*Note: It is recommended to use environment variables or a secure configuration method to manage your credentials, rather than hardcoding them.*

## API Coverage

This client aims to provide comprehensive coverage of the Sumsub API. The following modules are currently implemented:

*   **Applicants**: Create and retrieve applicant data.
*   **Applicant Actions**: Create, retrieve, and manage applicant actions (e.g., document uploads, identity checks).
*   **Business Verification (KYB)**: Create company applicants, link beneficiaries, and manage company data.
*   **Transaction Monitoring (KYT)**: Submit, delete, and bulk-import transactions for risk analysis.
*   **Travel Rule**: Initiate SDKs, patch transactions, and confirm wallet ownership.
*   **Miscellaneous**: Check API health and retrieve audit trail events.

## Error Handling

All API methods return a `Result<T, SumsubError>`. `SumsubError` is a comprehensive enum that covers potential issues, including:
*   Network errors from `reqwest`.
*   Serialization/deserialization errors from `serde_json`.
*   API errors returned by Sumsub.

This makes it easy to handle failures gracefully.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License.
