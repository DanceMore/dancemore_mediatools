# Spec 0003: Fix `koditool` Tests

## Problem Statement
The test `test_select_random_episode_by_title` in `koditool/tests/kodi_helper_test.rs` is failing intermittently.
The failure occurs when the random selection chooses episode `102` instead of the expected `101`.
The assertion error is `Assertion failed: 102 == 101`.

## Research Findings
In `koditool/src/kodi_helper.rs`, the `select_random_episode_by_title` method uses a random seed for its RNG:

```rust
let mut seed_array = [0u8; 32];
let _ = rng().try_fill_bytes(&mut seed_array);
let mut rng = ChaCha12Rng::from_seed(seed_array);
```

This ensures that every call is non-deterministic. However, the test `test_select_random_episode_by_title` expects a deterministic outcome:

```rust
// Note: This test will always select episode 101 with our fixed seed
let result = client
    .select_random_episode_by_title("Friends")
    .await
    .unwrap();

assert_eq!(result.episode_id, 101);
```

The comment in the test is incorrect because `select_random_episode_by_title` does *not* use a fixed seed; it generates a new one every time.

## Proposed Fix
To make the test deterministic while maintaining randomness in production:

1.  **Modify `RpcClient`**: Add an optional `seed` field to `RpcClient` or allow passing a seed to `select_random_episode_by_title`. Since the method is intended for production use, adding a seed parameter to the method itself might be intrusive. A better approach is to allow setting a seed on the `RpcClient` instance.
2.  **Update `select_random_episode_by_title`**: Use the seed from `RpcClient` if it exists, otherwise generate a random one.
3.  **Update Tests**: When creating a `RpcClient` for testing, provide a fixed seed.

## Alternative Fix
Update the test to accept either episode `101` or `102`. However, this is less robust and doesn't solve the underlying issue of non-deterministic tests.

## Selected Strategy
Modify `RpcClient` to hold an optional seed:

```rust
pub struct RpcClient {
    pub config: Config,
    pub auth: Authorization,
    pub client: Client,
    pub seed: Option<[u8; 32]>,
}
```

And update `select_random_episode_by_title` to use this seed if present.
Update the test helper `test_client()` to initialize the client with a fixed seed.
