# Spec 0003: Jukeingest Hardening

## Problem Statement
`jukeingest` has several reliability and safety issues identified in the TODO list:
1.  **Playlist Directory Inclusion**: The ingest process currently scans the `playlists/` directory, potentially causing recursion or adding playlist files to playlists.
2.  **Dry-run Side Effects**: The `--dryrun` flag incorrectly updates the `last_run` timestamp, which messes up subsequent incremental runs.
3.  **Ambiguous Defaults**: The CLI allows running without explicit `--detect` (auto-incremental) or `--threshold` (manual days) flags, leading to unpredictable default behavior.

## Proposed Changes

### 1. Robust Playlist Exclusion
**Current Logic**:
```rust
!e.path().starts_with(&playlists_path)
```
This check might fail if `playlists_path` is not absolute or if `WalkDir` returns paths in a format that doesn't strictly prefix-match.

**New Logic**:
- Canonicalize `root_path` and `playlists_path` before starting the walk.
- Explicitly check if the entry's parent directory is the playlists directory.

### 2. Dry-run Purity
**Current Logic**:
`save_last_run_timestamp()` is called in the `else` block of `if cli.dryrun`, but the logic flow is slightly entangled.

**New Logic**:
Ensure `save_last_run_timestamp()` is *strictly* guarded by `!cli.dryrun`.
Verify that `process_directory` does not perform any write operations (it currently doesn't, but we should double-check).

### 3. Explicit Mode Requirement
**Current Logic**:
The CLI struct has a default value for `threshold_days` (30).
```rust
#[arg(short, long, default_value_t = 30)]
threshold_days: u32,
```

**New Logic**:
- Use `clap`'s `ArgGroup` to enforce that either `--detect` OR `--threshold` is provided.
- Remove the default value for `threshold_days` (make it `Option<u32>`) or keep it but enforce the group requirement.
- If the user provides neither, the program should exit with a help message.

## Implementation Details

### `Cargo.toml`
Ensure `clap` features are sufficient for `ArgGroup`.

### `src/main.rs`
- Update `Cli` struct to use `#[command(group(...))]`.
- Refactor `main` to handle the `Option` or group logic.
- Update `process_directory` to use `std::fs::canonicalize` for robust path comparison.

## Verification Plan
1.  **Test Playlist Exclusion**:
    - Create a dummy directory structure with a `playlists/` folder containing dummy files.
    - Run `jukeingest` and assert those files are NOT in the output.
2.  **Test Dry-run**:
    - Run with `--dryrun`.
    - Assert `.jukeingest/last_run` file is NOT created or modified.
3.  **Test Explicit Flags**:
    - Run without flags -> Expect Error/Help.
    - Run with `--detect` -> Success.
    - Run with `--threshold 10` -> Success.
