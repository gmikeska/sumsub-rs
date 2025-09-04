# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Overview

This is an unofficial, asynchronous Rust client for the [Sumsub API](https://docs.sumsub.com/reference/about-sumsub-api). The crate provides a type-safe interface for KYC/KYB verification workflows, transaction monitoring (KYT), and Travel Rule compliance.

**Key capabilities:**
- Create and manage applicants (individuals/companies)
- Upload documents and trigger verification checks
- Monitor transactions for AML compliance
- Implement Travel Rule requirements
- Strongly-typed models with automatic request signing

## Development Commands

### Build & Test
```bash
# Build the library and all tests
cargo build --all-targets

# Run all tests (requires env vars - see below)
cargo test

# Run tests with output
cargo test -- --nocapture

# Run ignored integration tests (hits live API)
cargo test -- --ignored
```

### Code Quality
```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Check for compile errors without building
cargo check --all-targets
```

### Environment Setup
The integration tests require Sumsub API credentials. Use dotenvy for local development:

```bash
# Create .env file
echo 'SUMSUB_APP_TOKEN=your_app_token' >> .env
echo 'SUMSUB_SECRET_KEY=your_secret_key' >> .env

# Tests will automatically load from .env if using dotenvy
```

## Architecture

The crate follows a modular client design:

```text
┌─────────────────────────────────────────────────────────┐
│                    Client (client.rs)                    │
│  - Holds app_token, secret_key                          │
│  - Contains reqwest::Client                             │
│  - Implements send_request() with HMAC signing          │
└────────────────────────┬────────────────────────────────┘
                         │
    ┌────────────────────┼────────────────────┐
    │                    │                    │
┌───▼─────┐     ┌────────▼──────┐    ┌───────▼──────┐
│ Models  │     │ Domain Modules│    │    Error     │
│         │     │               │    │              │
│ Structs │     │ - actions.rs  │    │ SumsubError  │
│ for req │     │ - kyb.rs      │    │ enum with    │
│ & resp  │     │ - misc.rs     │    │ reqwest &    │
│         │     │ - transactions│    │ serde_json   │
│         │     │ - travel_rule │    │ variants     │
└─────────┘     └───────────────┘    └──────────────┘
```

**Key components:**
- `Client`: Main entry point, handles authentication and HTTP communication
- `models.rs`: Core data structures (Applicant, CreateApplicantRequest, etc.)
- Domain modules: Type definitions for specific API areas
- `error.rs`: Unified error handling via `SumsubError`

## Authentication & API Requests

The client automatically handles Sumsub's HMAC-SHA256 authentication:

1. **Headers sent with each request:**
   - `X-App-Token`: Your application token
   - `X-App-Access-Ts`: Unix timestamp
   - `X-App-Access-Sig`: HMAC signature of `timestamp + method + path + body`

2. **Signature generation** (in `sign_request()`):
   ```
   HMAC-SHA256(secret_key, timestamp + HTTP_METHOD + path + request_body)
   ```

3. **Base URL**: Defaults to `https://api.sumsub.com` (production). For testing on testnet (per user rules), ensure your app token is configured for the test environment in the Sumsub dashboard.

## Testing

### Unit Tests
The crate includes basic unit tests that can run without API credentials:
```bash
cargo test --lib
```

### Integration Tests
Integration tests in `tests/api_tests.rs` interact with the live Sumsub API:
```bash
# Set environment variables first
export SUMSUB_APP_TOKEN="your_token"
export SUMSUB_SECRET_KEY="your_secret"

# Run integration tests (marked with #[ignore])
cargo test -- --ignored
```

**Test coverage includes:**
- Creating and fetching applicants
- API health status checks
- Applicant action operations
- Transaction data retrieval

### Missing Dependencies
Note: The test file references `uuid` which isn't in `Cargo.toml`. To fix:
```bash
cargo add uuid --dev
```

## Common API Operations

### Create an Applicant
```rust
use sumsub_api::client::Client;
use sumsub_api::models::CreateApplicantRequest;

let client = Client::new(app_token, secret_key);

let request = CreateApplicantRequest {
    external_user_id: "user-123".to_string(),
    ..Default::default()
};

let applicant = client.create_applicant(request, "basic-kyc-level").await?;
```

### Submit a Transaction for Monitoring
```rust
use sumsub_api::transactions::SubmitTransactionRequest;

let txn_request = SubmitTransactionRequest {
    // ... transaction details
};

let response = client.submit_transaction("applicant-id", txn_request).await?;
```

## API Endpoint Coverage

The client implements these Sumsub API areas:

- **Applicants**: create, retrieve, update
- **Actions**: document uploads, verification triggers, questionnaires
- **KYB**: company verification, beneficiary management
- **Transactions**: submit, delete, bulk import for KYT
- **Travel Rule**: SDK initialization, wallet ownership confirmation
- **Miscellaneous**: health checks, audit trail events

## Error Handling

All API methods return `Result<T, SumsubError>`:

```rust
match client.create_applicant(request, level).await {
    Ok(applicant) => println!("Created: {}", applicant.id),
    Err(SumsubError::Reqwest(e)) => eprintln!("Network error: {}", e),
    Err(SumsubError::Serde(e)) => eprintln!("JSON error: {}", e),
}
```

## Development Tips

1. **Rate Limiting**: Sumsub may rate-limit requests. Implement retry logic with exponential backoff for production use.

2. **Webhook Integration**: For production systems, implement webhook handlers to receive verification status updates instead of polling.

3. **Logging**: Add request/response logging for debugging (ensure you don't log sensitive data like the secret key).

4. **Pagination**: Some list endpoints support pagination - check Sumsub docs and add pagination parameters as needed.
