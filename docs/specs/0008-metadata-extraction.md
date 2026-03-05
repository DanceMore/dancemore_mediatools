# Spec 0008: Metadata Extraction for jukeingest (C2)

## Goal
Enhance `jukeingest` to extract metadata (Artist, Album, Title) from media files using the `id3` or `audiotags` crate.

## Plan
1. Add `id3` crate to `jukeingest/Cargo.toml`.
2. Update `process_directory` to attempt to read tags for each file.
3. If tags are found, format the playlist entry as `Artist - Title.ext` or similar (configurable).
4. Fall back to the filename if no tags are present.

## Verification
- Run ingest on a directory with tagged MP3s.
- Verify the output playlist contains the expected metadata-based strings.
