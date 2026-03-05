# Spec 0002: Project Roadmap

## Priority 1: Core System Integrity
- **Fix `koditool` Test Failure**: Resolve `test_select_random_episode_by_title` (Assertion failed: 102 == 101).
- **Cleanup Warnings**: Resolve compilation and clippy warnings across `jukeingest`, `dupehunter`, and `koditool`.

## Priority 2: `jukeingest` Bug Fixes
- **Exclude `playlists/`**: Ensure the ingest process correctly ignores the `playlists/` directory.
- **Dry-run Correction**: Prevent `dryrun` from updating the `last_run` timestamp.
- **Explicit Flags**: Update CLI to require `--detect` or `--threshold`, removing unsafe default behaviors.

## Priority 3: `tv_mode_web` Productionization
- **Full API Coverage**: Add integration tests for POST `/api/play/<user>` and GET `/api/status`.
- **Error Handling**: Verify system behavior when the Kodi RPC backend is unreachable.
- **Frontend Refinement**: Enhance the `index.html.j2` and `jukectl.html.j2` templates for better user feedback.

## Priority 4: SDLC Adherence
- **Documentation**: Ensure every new feature or fix is preceded by a specification in `docs/specs/`.
- **Validation**: Every change must be verified via `make test`.
