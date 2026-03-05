# Spec 0001: Web Verification Strategy

## Overview
This specification describes the current state of the `tv_mode_web` component and the strategy for verifying its integrity using Rust integration tests.

## Current Prototype Analysis

### Main Routes
| Path | Method | Description | Template / Response |
|------|--------|-------------|---------------------|
| `/` | GET | Main dashboard for TV Mode | `index.html.j2` |
| `/jukectl` | GET | Jukebox control interface | `jukectl.html.j2` |
| `/api/status` | GET | Current status of TV Mode & Media Server | JSON (`StatusResponse`) |
| `/api/users` | GET | List of configured users and shows | JSON (`UsersResponse`) |
| `/api/health` | GET | Simple API health check | JSON (`StatusResponse`) |

### Templates
- `index.html.j2`: Uses standard HTML/CSS. Displays TV mode controls.
- `jukectl.html.j2`: Displays jukebox channels and playback controls.

## Verification Strategy

### Goal
Ensure that all main routes are accessible (200 OK) and that the HTML templates render correctly with expected content.

### Method: Rust Integration Tests
We will use `rocket::local::asynchronous::Client` to perform integration testing. This allows us to:
1. Initialize a test instance of the Rocket application.
2. Dispatch requests to specific routes.
3. Assert the HTTP status code.
4. Inspect the response body for specific HTML strings or JSON fields.

### Test Cases
1. **Root Route (`/`)**:
   - Status: 200 OK
   - Content: Should contain "TV Mode" or similar dashboard identifiers.
2. **Jukectl Route (`/jukectl`)**:
   - Status: 200 OK
   - Content: Should contain "Jukebox" or channel list elements.
3. **Health Check (`/api/health`)**:
   - Status: 200 OK
   - Content: JSON with `status: "success"`.

### Implementation Plan
- Create `tv_mode_web/tests/web_integrity.rs`.
- Implement a helper to create a test `Rocket` instance.
- Implement `async` test functions for each case.
