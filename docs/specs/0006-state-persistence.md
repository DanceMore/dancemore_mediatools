# Spec 0006: State Persistence & Recovery (A2)

## Goal
Ensure `tv_mode_web` recovers its "TV Mode Active" and "Sleep Timer" state after a process restart.

## Plan
1. Create a `persistent_state.json` file in the configuration directory.
2. Update `AppState` to include a `save_to_disk()` method.
3. During `app_state::initialize`, attempt to load this file.
4. If found, re-initialize the `tv_mode` status and sleep timer with the saved timestamps.

## Verification
- Start TV Mode.
- Manually restart the server.
- Verify through `/api/status` that TV Mode is still "Active" and the timer is correctly counting down from the original start time.
