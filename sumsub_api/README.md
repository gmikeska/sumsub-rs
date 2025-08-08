# Sumsub API

A Rust crate for interacting with the [Sumsub API](https://docs.sumsub.com/reference/about-sumsub-api).

This crate provides a client for the Sumsub API, allowing you to
perform actions such as creating applicants, uploading documents, and
getting verification results.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
sumsub_api = { git = "https://github.com/example/sumsub_api" } # Replace with the actual URL
```

## Example

Here is a basic example of how to use the `sumsub_api` crate to create an applicant and then retrieve their data:

```rust
use sumsub_api::client::Client;
use sumsub_api::models::CreateApplicantRequest;

#[tokio::main]
async fn main() {
    let app_token = "YOUR_APP_TOKEN".to_string();
    let secret_key = "YOUR_SECRET_KEY".to_string();

    let client = Client::new(app_token, secret_key);

    let request = CreateApplicantRequest {
        external_user_id: "some-unique-user-id".to_string(),
        ..Default::default()
    };

    let applicant = client
        .create_applicant(request, "basic-kyc-level")
        .await
        .unwrap();

    println!("Created applicant: {:?}", applicant);

    let fetched_applicant = client.get_applicant_data(&applicant.id).await.unwrap();

    println!("Fetched applicant: {:?}", fetched_applicant);
}
```

## Contributing

Contributions are welcome! Please feel free to submit a pull request.

## License

This project is licensed under the MIT License.
