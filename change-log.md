# Change Log

## [Unreleased] - 2026-06-13

### Added
- **Rust Backend Test Suite:** Added unit and integration tests across `panoptic-core` (playback state formatting), `panoptic-cache` (AssetCache idempotency and isolation), `panoptic-provider-linux` (refactored metadata parser and added mock tests), `panoptic-server` (auth callback error, success, missing parameters, and health checks), and `panoptic-gui` (AppSettings serialization and defaults).
- **React Frontend Component Tests:** Integrated Vitest and React Testing Library in `crates/ui/panoptic-gui`. Wrote component rendering, navigation tab switching, and custom Client ID submission tests.
- **Tauri Mock Testing Environment:** Created `setupTests.ts` and `vitest.config.ts` to mock Tauri's `@tauri-apps/api/core` and `@tauri-apps/api/event` interfaces, allowing React components to be tested in a headless JSDOM environment.
- **GitHub Actions CI Workflow (`.github/workflows/ci.yml`):** Established a workflow running on Ubuntu, installing required APT developer libraries for Tauri v2 compilation, setting up stable Rust and Node.js, checking formatting (`cargo fmt --check`), running Clippy (`cargo clippy`), and executing both backend (`cargo test`) and frontend (`npm run test`) test suites on every push and pull request.
- **Multi-Platform Build Script (`build.py`):** Added a python-based build automation utility to trigger native Linux builds and cross-compile/package Windows binaries via `cargo-xwin` or containerized Docker builds. Added a `--bundle` flag to optionally produce packaged installers (e.g., version-aware NSIS `.exe` installers) by installing and running `nsis`/`makensis` inside the container or host.
- **Axum AppState & Router State Sharing:** Refactored `panoptic-server` to share a single unified `AppState` containing both auth sender and playback state receiver, allowing multiple endpoints to access distinct state variables.
- **Current Track HTTP Endpoint:** Added `GET /current-track` at `http://127.0.0.1:3000/current-track` to return the formatted track string as plain text (`text/plain`).
- **Spotify PKCE Authentication:** Implemented Authorization Code Flow with PKCE (Proof Key for Code Exchange) to secure authentication on the desktop client without hardcoding or packaging a Client Secret.
- **PKCE Cryptography Module:** Added `crates/ui/panoptic-gui/src-tauri/src/engine/pkce.rs` for generating high-entropy code verifiers and SHA-256 code challenges.
- **AppSettings Manager:** Added `crates/ui/panoptic-gui/src-tauri/src/engine/settings.rs` to load and save Client ID, Access Token, and Refresh Token to `settings.json` within the app's configuration directory.
- **Token Refresh Cycle:** Automatic token refreshing (via the refresh token and client ID) on HTTP `401 Unauthorized` responses in the main orchestrator loop.
- **Tauri Commands:**
  - `get_spotify_client_id`
  - `set_spotify_client_id`
  - `unlink_spotify`
  - `get_spotify_status`
- **React Settings UI:** A new configuration form in the **Auth** panel of `App.tsx` allowing users to view, edit, and save their custom Spotify Client ID.
- **Wayland Compatibility Fix:** Programmatically set `WEBKIT_DISABLE_DMABUF_RENDERER=1` on Linux in `main.rs` to prevent WebKitGTK DMA-BUF rendering protocol crashes on Wayland (e.g. Hyprland).
- **Live Playback Event Updates:** Implemented Tauri event emission (`playback_update`) from the Rust engine orchestrator loop to React, with a listener in `App.tsx` updating playback details in real time.
- **UI & Theme Overhaul:** Integrated high-end dark mode aesthetics, resolving transparent background and contrast issues on Wayland, custom scrollbars, stylized input fields, Outfit & Inter premium typography, and glowing hover states.
- **Output Templating Preview & Guide:** Added a live-updating preview box and a responsive guide displaying all available placeholders (`{title}`, `{artist}`, `{album}`, `{progress_ms}`, `{duration_ms}`) below the Output Template editor.
- **Real-time Live CSS Preview:** Merged the CSS Stylesheet editor directly side-by-side with the Live Overlay preview under the "Live Overlay" tab. Created a dynamic stylesheet injection mechanism (`<style>` tag) in `App.tsx` so styling changes apply immediately in real-time.
- **Highly Configurable Overlay DOM:** Expanded the HTML structure of the Live Overlay preview with descriptive, specific class names prefixed with `panoptic-overlay-` (e.g. `.panoptic-overlay-card`, `.panoptic-overlay-track-title`, `.panoptic-overlay-time-display`) for exhaustive, modular styling capabilities.

- **Window Close Interception:** Intercepts `tauri::WindowEvent::CloseRequested` to hide the window instead of closing it, allowing the background engine to remain active and the window to be reopened from the system tray.
- **Detailed CSS Variables:** Declared self-explanatory custom CSS variables (e.g. `--panoptic-overlay-card-background`, `--panoptic-overlay-album-art-width`, `--panoptic-overlay-track-title-text-color`) in `:root` of the default Live CSS template.

### Changed
- **Linux MPRIS Metadata Parser:** Refactored `MprisMetadataParser` to extract property parsing into `parse_metadata_map`, separating DBus connection querying from parsing logic to enable unit testing.
- **Dependencies:** Added `rand`, `sha2`, `base64`, and `reqwest` to `panoptic-gui/src-tauri/Cargo.toml`.
- **Axum Callback handler:** Changed `spotify_callback` in `panoptic-server` to propagate the auth code via `AuthState::Authenticating` instead of using mock credentials.
- **API Client:** Modified `SpotifyApiClient` to return `reqwest::Error` on failures so `WebFallbackEngine` and `EngineOrchestrator` can detect `401 Unauthorized` states.
- **Transitive Pinning:** Locked `time` crate to `0.3.47` in `Cargo.lock` to fix E0119 trait conflict error.
- **Window Resizability & Limits:** Configured Tauri's main window to be resizable with a minimum width/height limit of `800x550` in `tauri.conf.json`.
- **Fluid Layout & Scroll Containment:** Added styling for `#root` in `index.css` to prevent mounting containers from collapsing. Set `.content` to `overflow: hidden` globally, and added a `.view-pane-scrollable` class to standard settings views (storage, auth, output).
- **Vertical Split Layout:** Redesigned the Live Overlay tab in `App.tsx` to stack components vertically (Live Preview on top, wide CSS Editor on the bottom), giving the code editor the full width of the pane.

### Fixed
- **Docker Missing Windows Linker (`link.exe`):** Configured `build.py` to auto-install `cargo-xwin` inside the container (cached on the host via volume mount) and passed `--runner cargo-xwin` to the Tauri builder, ensuring the compiler toolchain is updated *before* installing `cargo-xwin` to satisfy its minimum rustc requirements (1.89+). Configured the build commands to pass `--no-bundle` when cross-compiling to the MSVC target, producing the `.exe` binary successfully while bypassing Windows installer packaging errors on non-Windows hosts.
- **Docker Missing Resource Compiler (`llvm-rc`):** Configured `build.py` to run `apt-get install -y llvm clang lld` inside the container to provide the necessary compiler tools and the `llvm-rc` binary required for compiling Windows resource files (`.rc`).
- **Docker Compiler Version Mismatch:** Added `rustup update stable && rustup default stable` prior to compilation inside the container in `build.py` to upgrade the container's Rust version from 1.85.1 to the latest stable release and activate it as the default toolchain, satisfying Cargo.lock dependency requirements (e.g., `time` and `serde_with` requiring Rust 1.88.0+).
- **Docker Compiler Target Error:** Integrated `rustup target add x86_64-pc-windows-msvc` into the container compile sequence in `build.py` to ensure the MSVC compilation target is available prior to launching the Tauri builder.
- **Docker Build File Permissions (EACCES):** Configured the Docker build command in `build.py` to run inside a Python `try-finally` block to automatically restore host file ownership (`chown -R`) for all generated frontend and backend compile artifacts even on early compile failures, avoiding permission denied errors on successive local host builds.
- **Progress Bar Stalling:** Added defensive bounds checking to the React `progressPercent` calculation to prevent `NaN%` styles when `duration_ms` is `0` or undefined.
- **MPRIS Position Fallback:** Safely default the MPRIS `Position` query in `mpris/parser.rs` to 0 on failure to prevent the entire metadata extraction from aborting when players are paused or don't support position queries.
- **Progress Jitter / Timer Fighting:** Implemented a monotonic timer interpolation loop (~33fps, 30ms) based on the elapsed time since the last backend event poll baseline. Bound the current time text component to the interpolated progress value. This completely eliminates progress bar stuttering and visual jump-back.
- **CSS Naming Collision:** Renamed the overlay preview container class from `.overlay-container` to `.panoptic-overlay-preview-container` to avoid conflicts with `.overlay-container` (`height: 100vh`) defined in `overlay.css`, resolving the bug where the preview occupied the entire window and hid the CSS editor.
- **CSS Editor Sizing / Blank Space:** Resolved the bug where the CSS editor textarea collapsed to its minimum height and left a massive blank space at the bottom of the window, forcing the editor to correctly stretch to fill all remaining vertical space.
