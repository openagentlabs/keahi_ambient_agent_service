# Firestore Integration Tests

This document describes how to set up and run integration tests against a real Firestore database.

## Overview

The Firestore integration tests validate that the database repositories work correctly with a real Firestore instance. These tests are separate from the mock repository tests and are designed to ensure that:

1. Data is actually written to and read from Firestore
2. All CRUD operations work correctly
3. Queries and filters function as expected
4. Error handling works properly with real database operations

## Prerequisites

### 1. Google Cloud Project
- You need a Google Cloud project with Firestore enabled
- The project should have a Firestore database created

### 2. Service Account
- Create a service account with Firestore permissions
- Download the service account key JSON file
- Set the `GOOGLE_APPLICATION_CREDENTIALS` environment variable

### 3. Environment Setup
```bash
# Set the path to your service account key file
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/your/service-account-key.json

# Verify the credentials are accessible
gcloud auth activate-service-account --key-file=$GOOGLE_APPLICATION_CREDENTIALS
```

## Running the Tests

### Option 1: Using the Script
```bash
# Make sure the script is executable
chmod +x scripts/run_firestore_tests.sh

# Run the tests
./scripts/run_firestore_tests.sh
```

### Option 2: Direct Cargo Command
```bash
# Run all Firestore integration tests
cargo test --test firestore_integration_tests -- --ignored

# Run a specific test
cargo test --test firestore_integration_tests test_firestore_client_repository_integration -- --ignored
```

### Option 3: Run with Regular Tests (Not Recommended)
```bash
# Run all tests including Firestore integration tests
cargo test -- --ignored
```

## Test Coverage

The integration tests cover:

### Client Repository
- ✅ Creating clients with and without room association
- ✅ Retrieving clients by ID and auth token
- ✅ Updating client information
- ✅ Listing clients with pagination
- ✅ Deleting clients
- ✅ Validating authentication
- ✅ Checking client existence

### Terminated Room Repository
- ✅ Creating terminated room records
- ✅ Retrieving terminated room information
- ✅ Listing terminated rooms with pagination
- ✅ Checking if a room was terminated
- ✅ Querying by date range

### Room Created Repository
- ✅ Creating room creation records
- ✅ Retrieving room creation information
- ✅ Listing room creation records with pagination
- ✅ Checking if a room was created
- ✅ Querying by date range

### Repository Factory
- ✅ Creating all repository types
- ✅ Verifying repository instantiation

## Test Data

The tests use unique identifiers to avoid conflicts:
- Client IDs: `test_client_{uuid}`
- Room IDs: `test_room_{uuid}`
- Auth tokens: `test_token_{uuid}`
- Room UUIDs: Generated UUIDs

## Error Handling

The tests include proper error handling for:
- Missing credentials
- Network connectivity issues
- Firestore permission errors
- Invalid data formats

## Cleanup

The tests are designed to clean up after themselves:
- Created clients are deleted after testing
- Test data uses unique identifiers to avoid conflicts
- No permanent data is left in the database

## Configuration

The tests use the default configuration from `Config::default()`. If you need to customize the Firestore connection, you can modify the test setup in `tests/firestore_integration_tests.rs`.

## Troubleshooting

### Common Issues

1. **Credentials not found**
   ```
   Error: GOOGLE_APPLICATION_CREDENTIALS environment variable is not set
   ```
   Solution: Set the environment variable to your service account key file path.

2. **Permission denied**
   ```
   Error: Firestore permission error
   ```
   Solution: Ensure your service account has Firestore read/write permissions.

3. **Network connectivity**
   ```
   Error: Connection timeout
   ```
   Solution: Check your internet connection and firewall settings.

4. **Project not found**
   ```
   Error: Project ID not found
   ```
   Solution: Verify your project ID in the configuration.

### Debug Mode

To run tests with more verbose output:
```bash
RUST_LOG=debug cargo test --test firestore_integration_tests -- --ignored
```

## Security Notes

- Never commit service account keys to version control
- Use environment variables for sensitive configuration
- Consider using temporary credentials for testing
- Regularly rotate service account keys

## Performance

- Tests may take longer than mock tests due to network latency
- Consider running tests in parallel if you have multiple test databases
- Monitor Firestore usage to avoid hitting quotas 