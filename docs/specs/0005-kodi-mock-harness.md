# Spec 0005: Kodi RPC Mock Harness

## Goal
Establish a high-fidelity mock environment for the Kodi JSON-RPC API. This allows `Jules` to verify "bulletproof" behavior (timeouts, error handling, state recovery) without needing a real Kodi instance.

## Technical Architecture

### 1. The Mock Server (`mockito`)
We will use the `mockito` crate to spin up a local HTTP server that intercept requests to `HOST/jsonrpc`.

### 2. Mocking Strategy: Behavioral Injection
Instead of static JSON files, the harness will support **Behavioral Injection**. We should be able to configure the mock to:
- **`return_success(Value)`**: Return a standard 200 OK with valid JSON.
- **`return_error(Status, Value)`**: Return a specific HTTP error code (e.g., 401 Unauthorized, 500 Internal Error).
- **`simulate_timeout(Duration)`**: Delay the response to verify that `tv_mode_web`'s `RPC_TIMEOUT` (5s) works correctly.
- **`return_malformed()`**: Return invalid JSON to test parser resilience.
- **`simulate_network_failure()`**: Close the connection abruptly.

### 3. API Surface to Mock
The harness must implement handlers for these specific Kodi methods:
- `VideoLibrary.GetTVShows`
- `VideoLibrary.GetEpisodes`
- `VideoLibrary.GetEpisodeDetails`
- `Player.GetActivePlayers`
- `Player.Open`
- `Player.Stop`

## Usage in Tests

### Example Test Case
```rust
#[rocket::async_test]
async fn test_rpc_timeout_handling() {
    let mut server = mockito::Server::new_async().await;
    let mock = server.mock("POST", "/jsonrpc")
        .with_delay(Duration::from_secs(10)) // Longer than our 5s timeout
        .create_async().await;

    let client = create_test_client_with_url(&server.url()).await;
    let response = client.get("/api/status").dispatch().await;

    // Verification: Status should be 200 but message should indicate timeout/error
    let body: StatusResponse = response.into_json().await.unwrap();
    assert_eq!(body.status, "error");
    assert!(body.message.contains("timed out"));
}
```

## Implementation Plan
1.  **Harness Module**: Create `tv_mode_web/tests/harness/mod.rs` to encapsulate mock server setup.
2.  **Mock Builders**: Implement helper functions to generate standard Kodi JSON responses (e.g., `mock_tv_shows_list()`).
3.  **Integration**: Update `web_integrity.rs` to use this harness for all API-level tests.

## Why this is "Jules-Proof"
This design provides a clear, programmatic way for an LLM agent to "break" the system and then "fix" it. By providing the harness, we ensure that Jules' fixes are verified against real-world failure conditions, not just happy-path scenarios.
