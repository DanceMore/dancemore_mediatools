# Spec 0011: Volume Control RPC (B4)

## Goal
Implement volume adjustment and mute status control in `koditool`.

## RPC Methods
1.  **`Application.SetVolume`**: Sets volume to a specific integer (0-100).
2.  **`Application.SetMute`**: Toggles mute status.
3.  **`Application.GetProperties`**: Checks current volume and mute state.

## Plan
1. Add `set_volume(u32)` method to `RpcClient`.
2. Add `toggle_mute()` method to `RpcClient`.
3. Add `get_properties(Vec<String>)` method for general application-level state.

## Verification
- Mock these calls and verify the `rpc_call` body contains the correct JSON.
