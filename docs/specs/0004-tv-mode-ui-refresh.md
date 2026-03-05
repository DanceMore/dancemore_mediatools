# Spec 0004: TV Mode UI Refresh

## Goal
Transform the `tv_mode_web` interface from a basic prototype into a polished, modern, and reactive control panel for the media system.

## Design Philosophy
- **Aesthetic**: "Dark Mode First", cinematic feel. Use deep grays/blacks (#121212) with high-contrast accent colors (Neon Green/Blue) for active states.
- **Tech Stack**: Keep it lightweight. No heavy JS framework (React/Vue/Angular). Use **HTMX** for interactivity and **Pico.css** (or a custom minimal CSS) for styling. This aligns with the Rust backend + Minijinja templates architecture.
- **Responsiveness**: Must work perfectly on mobile devices (the primary remote control) and desktop browsers.

## Key Features

### 1. Dashboard (`/`)
- **Hero Section**: Large, clear status indicator ("TV Mode Active" / "Idle").
- **User Grid**: 
    - Display users as large, touch-friendly cards.
    - Show the number of available shows for each user (from `show_mappings`).
    - **Interaction**: Tap card -> Opens "Play Options" modal (Sleep timer selection) or immediate play.
- **Sleep Timer Controls**:
    - Visible countdown if active.
    - Quick "Add 30m" or "Cancel" buttons.
- **Now Playing**:
    - If media is active, show the show/episode title (requires new API data).

### 2. Jukectl (`/jukectl`)
- **Queue Visualization**: Show the next 5 tracks.
- **Quick Actions**: "Skip", "Album Mode Toggle", "Tag Current Track".
- **Visual Feedback**: Buttons should show loading states and success/failure toasts.

### 3. Technical Changes

#### Backend (`src/routes/`)
- **New Endpoints**:
    - `GET /api/status/fragment`: Returns just the HTML fragment for the status bar (for HTMX polling).
    - `POST /api/play/<user>/fragment`: Returns the updated status fragment after starting playback.
- **Template Updates**:
    - Refactor `index.html.j2` to use HTMX attributes (`hx-get`, `hx-trigger`, `hx-target`).
    - Add `_layout.html.j2` base template for shared CSS/JS resources.

#### Frontend
- **Libraries**:
    - HTMX (via CDN or local asset).
    - FontAwesome (or similar icon set) for visual polish.

## Migration Plan
1.  **Refactor Templates**: Create the base layout and move existing logic to it.
2.  **Integrate HTMX**: Add the library and convert the "Stop" button to use `hx-post="/api/stop"`.
3.  **Redesign Dashboard**: Implement the card grid layout.
4.  **Add Polling**: Use `hx-trigger="every 5s"` on the status container to auto-refresh the state without page reloads.

## Verification
- **Visual Check**: Does it look good on mobile?
- **Functional Check**: Does the status update automatically when another client changes the state?
- **Performance**: Does the page load instantly (<100ms)?
