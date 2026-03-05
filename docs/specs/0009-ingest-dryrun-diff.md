# Spec 0009: Ingest Dry-run Diff Reporting (C3)

## Goal
Improve the `--dryrun` output of `jukeingest` to show exactly what *would* be added or removed from the playlist.

## Plan
1. Before performing the walk, read the current playlist file.
2. Store the current file list in a `HashSet`.
3. After the new walk is complete, perform a `difference()` between the new list and the old list.
4. Print a colorized diff:
   - `[+] File to be added` (Green)
   - `[-] File to be removed` (Red - *optional if removals are supported*)
5. Show a summary: "Dry run: 12 files would be added to <playlist_path>".

## Verification
- Run `jukeingest --dryrun` on a directory with 5 new files.
- Assert that exactly those 5 files are shown with a `[+]` prefix.
