# Master Work Breakdown Structure (WBS)

This document tracks the roadmap to a "Bulletproof Production Service." Each item is a candidate for a Jules Task.

## Track A: `tv_mode_web` Resilience & UX (20 Tasks)
- [ ] A1: Mock Harness Implementation (Spec 0005)
- [ ] A2: State Persistence (JSON/File-based recovery)
- [ ] A3: RPC Timeout & Circuit Breaker (Prevent Kodi hangs)
- [ ] A4: Graceful Degradation UI (Offline Mode indicators)
- [ ] A5: User Mapping Validator (Fail-fast on config typos)
- [ ] A6: Atomic UI: User Selection Grid (Touch-optimized)
- [ ] A7: Atomic UI: Sleep Timer Countdown (Live updates)
- [ ] A8: Atomic UI: Now Playing Banner (Kodi Sync)
- [ ] A9: API Rate Limiting (Prevent family "remote wars")
- [ ] A10: Authentication Layer (Simple PIN or User selection)
- [ ] A11: Structured Logging (`tracing` integration)
- [ ] A12: Prometheus Metrics Endpoint (Track uptime/errors)
- [ ] A13: Health Check expansion (Check sub-dependencies)
- [ ] A14: Multi-instance State Sync (Redis or Shared File)
- [ ] A15-A20: [TBD - Frontend Refinements]

## Track B: `koditool` Core Expansion (15 Tasks)
- [ ] B1: Fix Assertion Failure (102 != 101)
- [ ] B2: Music Library Support (GetAlbums/GetSongs)
- [ ] B3: PVR/Live TV Support (GetChannels)
- [ ] B4: Volume Control RPC (Mute/Vol Up/Vol Down)
- [ ] B5: Connection Pooling (Reqwest Client reuse)
- [ ] B6: Retry Logic (Exponential backoff for RPC)
- [ ] B7: RPC Batching (Combine status requests)
- [ ] B8-B15: [TBD - API Coverage]

## Track C: `jukeingest` & `dupehunter` Hardening (15 Tasks)
- [ ] C1: Ingest Hardening (Spec 0003)
- [ ] C2: Metadata Extraction (Extract Artist/Album from ID3)
- [ ] C3: Ingest Dry-run Report (Generate diff of changes)
- [ ] C4: Dupehunter Fuzzy Matching (Detect similar filenames)
- [ ] C5: Dupehunter Safe-Delete (Trash can instead of rm)
- [ ] C6-C15: [TBD - Data Integrity]

## Track D: Infrastructure & Deployment (10 Tasks)
- [ ] D1: Docker Multi-stage Optimization
- [ ] D2: GitHub Actions: Matrix Testing (OSX/Linux/Windows)
- [ ] D3: Automated Changelog Generation
- [ ] D4: Configuration Hot-Reload (Watch yml files)
- [ ] D5-D10: [TBD - DevOps]
