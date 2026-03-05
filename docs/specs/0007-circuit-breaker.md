# Spec 0007: RPC Timeout & Circuit Breaker (A3)

## Goal
Prevent `tv_mode_web` from hanging or spamming a failing Kodi instance.

## Plan
1. Wrap all `rpc_call` invocations in a standard 5s timeout.
2. Implement a "Circuit Breaker" pattern in the `scheduler`.
3. If Kodi fails 5 times consecutively, enter "Backoff Mode" (wait 5 mins before checking again).
4. Update the API status to report "Media Server Unreachable (Retrying in X mins)".

## Verification
- Use the Kodi Mock Harness to simulate connection timeouts.
- Assert that the service remains responsive and properly enters the backoff state.
