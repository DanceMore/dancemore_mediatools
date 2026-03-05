# Spec 0010: Music Library RPC Support (B2)

## Goal
Expand `koditool`'s `RpcClient` to support basic music library operations.

## Methods to Implement
1.  **`AudioLibrary.GetArtists`**: Returns a list of artists.
2.  **`AudioLibrary.GetAlbums`**: Returns albums, optionally filtered by artist.
3.  **`AudioLibrary.GetSongs`**: Returns songs, optionally filtered by album.

## Plan
1. Add new request structs for these methods.
2. Implement corresponding public methods in `RpcClient`.
3. Add mock cases to the mock harness research file.

## Verification
- Add new test cases to `koditool` or `tv_mode_web` using the mock harness to verify the JSON parsing.
