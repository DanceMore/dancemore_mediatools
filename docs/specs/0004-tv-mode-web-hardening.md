# Spec 0004: tv_mode_web Production Hardening

## Goal
Transform `tv_mode_web` into a "bulletproof" service that provides reliable, predictable control for family members. Reliability and clear error communication take precedence over visual flair.

## Core Pillars

### 1. Resilience & Error Handling
- **RPC Robustness**: The system must never hang if the Kodi backend is slow or down. Implement strict timeouts (already started, but needs exhaustive testing).
- **Graceful Degradation**: If the media server is unreachable, the UI should display a "Media Server Offline" warning but still allow the user to see the configuration or try to reconnect.
- **Panic-Free**: Ensure no `unwrap()` calls exist in request handlers. Every error must be mapped to a user-friendly message.

### 2. State Consistency & Persistence
- **State Recovery**: Investigate/implement a simple file-based state persistence (e.g., `state.json`) so that if the server restarts, the TV Mode and Sleep Timer remain active.
- **Idempotency**: Requests like `/api/play` should be idempotent. If "Play" is clicked twice, it shouldn't cause a double-trigger of the scheduler.

### 3. Exhaustive Verification (Jules-Friendly)
- **Kodi Mocking**: Use `mockito` or a similar crate in `tests/` to simulate the Kodi JSON-RPC API.
- **Integration Test Scenarios**:
    - **Scenario A**: Kodi is down -> API returns 200 but with a clear "Media Server Unreachable" JSON/HTML status.
    - **Scenario B**: User starts TV Mode -> Restart `tv_mode_web` -> TV Mode is still "Active" (Persistence check).
    - **Scenario C**: Sleep timer expires during a network outage -> System handles it correctly when connectivity returns.

### 4. Simplified "Remote Control" UI
- **Large Targets**: High-contrast, large buttons for mobile use.
- **Feedback Loop**: Every button press must show immediate "Working..." or "Success/Fail" feedback to prevent double-tapping.
- **No-JS Fallback**: Ensure core functions (Play/Stop) work via standard HTML forms if a browser has JS issues.

## Implementation Plan
1.  **Add `mockito` to `dev-dependencies`** for advanced RPC mocking.
2.  **Refactor `app_state::initialize`** to load/save state from a local file.
3.  **Expand `web_integrity.rs`** to include the failure scenarios listed above.
4.  **Simplify Templates**: Focus on clear status labels and large, reliable buttons.

## Verification
- `make test` must pass with 100% reliability.
- Simulated network failures must not crash the service.
