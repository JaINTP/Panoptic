<p align="center">
  <img src="assets/panoptic-readme-header.png" alt="Panoptic" />
</p>

<p align="center">
  <strong>A modular, cross-platform streaming toolkit for overlays, data bridges, and live integrations.</strong>
</p>

<p align="center">
  <a href="https://github.com/JaINTP/Panoptic/releases"><img alt="Release" src="https://img.shields.io/github/v/release/JaINTP/Panoptic?style=flat-square&color=8b5cf6" /></a>
  <a href="https://github.com/JaINTP/Panoptic/actions"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/JaINTP/Panoptic/release.yml?style=flat-square&label=release" /></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-8b5cf6?style=flat-square" /></a>
  <a href="https://discord.gg/psBjVfq663"><img alt="Discord" src="https://img.shields.io/badge/discord-join-7289da?style=flat-square&logo=discord" /></a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-linux%20%7C%20windows-0d9488?style=flat-square" />
</p>

---

Panoptic is a lightweight, always-on desktop toolkit that provides modular streaming utilities - starting with a **Now Playing** engine that captures real-time media metadata and pipes it to OBS overlays, status bars, and external integrations. It runs in the system tray, stays out of your way, and is designed to grow with additional tools over time.

## Features

- **Native Media Detection** - reads directly from **MPRIS/D-Bus** on Linux and **SMTC (System Media Transport Controls)** on Windows. Zero browser extensions, zero polling hacks.
- **Spotify Web API Fallback** - when native providers can't reach the player (e.g. Spotify on web), Panoptic falls back to the Spotify Web API with automatic **PKCE authentication** and **token refresh**.
- **Live Overlay Preview** - a built-in, fully styleable "Now Playing" card with album art, track title, artist, and an interpolated progress bar (∼33 fps, jitter-free).
- **Custom CSS Theming** - edit the overlay stylesheet in a side-by-side editor with instant live preview. Every element uses descriptive `panoptic-overlay-*` class names and CSS variables for maximum control.
- **Output Templating** - define a format string (e.g. `Now Playing: {title} by {artist}`) and Panoptic writes the rendered result to `~/.config/panoptic/current_track.txt` every second, ready for OBS text sources.
- **HTTP API** - a local Axum server on `http://127.0.0.1:3000` exposes:
  - `GET /current-track` - formatted track string (plain text)
  - `GET /health` - health check
  - `GET /callback` - Spotify OAuth redirect handler
- **System Tray Integration** - minimises to tray on close. Right-click for Settings or Quit. The engine keeps running in the background.
- **Cross-Platform Builds** - ships as AppImage/`.deb` on Linux and NSIS `.exe`/`.msi` on Windows via automated GitHub Actions releases.

## Architecture

Panoptic is a **Tauri 2** application (React + Vite frontend, Rust backend) structured as a Cargo workspace with a modular crate layout designed for adding new tools alongside the existing media engine:

```
Panoptic/
├── crates/
│   ├── panoptic-core/          # Shared models (PlaybackState, AuthState) & MediaProvider trait
│   ├── panoptic-cache/         # Thread-safe asset cache (DashMap + UUID)
│   ├── audio/
│   │   ├── panoptic-provider-linux/    # MPRIS/D-Bus via zbus
│   │   ├── panoptic-provider-windows/  # SMTC via windows-rs
│   │   └── panoptic-provider-web/      # Spotify Web API fallback
│   ├── services/
│   │   └── panoptic-server/    # Axum HTTP server (track endpoint, OAuth callback, health)
│   └── ui/
│       └── panoptic-gui/       # Tauri app
│           ├── src/            # React frontend (Vite + TypeScript)
│           └── src-tauri/      # Rust backend (engine orchestrator, PKCE, settings)
├── build.py                    # Multi-platform build automation script
├── change-log.md
└── .github/workflows/
    └── release.yml             # CI/CD: build & publish releases
```

### Engine Orchestrator

The core loop in [`orchestrator.rs`](crates/ui/panoptic-gui/src-tauri/src/engine/orchestrator.rs) runs a priority chain every second:

1. **Native provider** - attempt MPRIS (Linux) or SMTC (Windows) fetch
2. **Web fallback** - if native fails, poll Spotify Web API
3. **Token refresh** - on `401 Unauthorized`, automatically refresh the access token via PKCE
4. **Template render** - apply the user's output template to the `PlaybackState`
5. **File write** - persist to `~/.config/panoptic/current_track.txt`
6. **Event emit** - push `playback_update` Tauri event to the React frontend

## Installation

### Pre-built Releases (Recommended)

Download the latest installer for your platform from the [Releases](https://github.com/JaINTP/Panoptic/releases) page:

| Platform | Artifact | Notes |
|----------|----------|-------|
| **Linux** | `.AppImage`, `.deb` | AppImage is portable, `.deb` for Debian/Ubuntu |
| **Windows** | `.exe` (NSIS installer), `.msi` | NSIS installer is recommended |

### Arch Linux (PKGBUILD)

A [`PKGBUILD`](pkg/arch/PKGBUILD) is included for Arch Linux users:

```bash
cd pkg/arch
makepkg -si
```

This builds from the release tarball, installs the binary to `/usr/bin/panoptic`, registers a `.desktop` entry, and places icons into the hicolor theme.

### Building from Source

#### Prerequisites

- **Rust** ≥ 1.88 (stable)
- **Node.js** ≥ 22
- **npm** (bundled with Node.js)

##### Linux-specific

```bash
# Debian/Ubuntu
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
  patchelf libdbus-1-dev pkg-config libssl-dev \
  libgtk-3-dev libsoup-3.0-dev javascriptcoregtk-4.1

# Arch Linux
sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3 librsvg \
  patchelf dbus openssl gtk3 libsoup3
```

##### Windows-specific

- Visual Studio Build Tools with the **C++ desktop development** workload
- [NSIS](https://nsis.sourceforge.io/) (for bundled installers)

#### Build Steps

```bash
# Clone
git clone https://github.com/JaINTP/Panoptic.git
cd Panoptic

# Install frontend dependencies
cd crates/ui/panoptic-gui
npm install

# Development mode (hot-reload)
npx tauri dev

# Production build (native platform)
npx tauri build
```

#### Using the Build Script

The included [`build.py`](build.py) automates multi-platform builds:

```bash
# Build for Linux only
python build.py --linux

# Build for Linux with bundled installers (AppImage, .deb)
python build.py --linux --bundle

# Build for Windows (cross-compile via cargo-xwin on Linux host)
python build.py --windows --win-method local

# Build for Windows via Docker container
python build.py --windows --win-method docker --bundle

# Build everything
python build.py --all --bundle
```

## Usage

### First Launch

1. Launch Panoptic - it starts in the system tray.
2. Right-click the tray icon → **Settings** to open the configuration window.

### Spotify Authentication

Panoptic uses **PKCE (Proof Key for Code Exchange)** - no client secret is stored or shipped.

1. Open **Settings** → **Auth** tab.
2. *(Optional)* Enter your own Spotify **Client ID** if you prefer to use your own app registration. A default ID is provided for convenience.
3. Click **Link Spotify** - your browser opens the Spotify authorization page.
4. Approve the permissions - you're redirected back to `http://127.0.0.1:3000/callback` and the token exchange completes automatically.
5. The auth status indicator turns green when linked.

To unlink, click **Unlink Spotify** - this clears the stored tokens immediately.

### Output Template

Navigate to the **Output** tab and customise the format string. Available placeholders:

| Placeholder | Description | Example / Range |
|-------------|-------------|-----------------|
| `{title}` | Track title | `Resonance` |
| `{artist}` | Artist name | `Home` |
| `{album}` | Album name | `Odyssey` |
| `{progress}` | Smart progress (formatted) | `3:04` or `1:05:04` |
| `{duration}` | Smart duration (formatted) | `4:12` or `1:05:20` |
| `{progress_h}` | Progress hours (unpadded) | `0` or `1` |
| `{progress_m}` | Progress minutes of current hour (padded) | `00` - `59` |
| `{progress_s}` | Progress seconds of current minute (padded) | `00` - `59` |
| `{progress_m_raw}` | Progress minutes of current hour (unpadded) | `0` - `59` |
| `{progress_s_raw}` | Progress seconds of current minute (unpadded) | `0` - `59` |
| `{progress_m_total}` | Total progress minutes (unpadded) | `65` |
| `{progress_m_total_padded}` | Total progress minutes (padded) | `05` or `65` |
| `{progress_s_total}` | Total progress seconds (unpadded) | `3909` |
| `{progress_ms}` | Progress in milliseconds (raw) | `165000` |
| `{duration_h}` | Duration hours (unpadded) | `0` or `1` |
| `{duration_m}` | Duration minutes of current hour (padded) | `00` - `59` |
| `{duration_s}` | Duration seconds of current minute (padded) | `00` - `59` |
| `{duration_m_raw}` | Duration minutes of current hour (unpadded) | `0` - `59` |
| `{duration_s_raw}` | Duration seconds of current minute (unpadded) | `0` - `59` |
| `{duration_m_total}` | Total duration minutes (unpadded) | `65` |
| `{duration_m_total_padded}` | Total duration minutes (padded) | `05` or `65` |
| `{duration_s_total}` | Total duration seconds (unpadded) | `3909` |
| `{duration_ms}` | Duration in milliseconds (raw) | `210000` |

**Default template:** `Now Playing: {title} by {artist}`

The rendered output is:
- Written to **`~/.config/panoptic/current_track.txt`** every second (great for OBS text file sources)
- Available via **`GET http://127.0.0.1:3000/current-track`** (great for custom widgets)

### Live Overlay

The **Live Overlay** tab provides a real-time preview of the "Now Playing" card alongside a CSS editor. Edit the stylesheet and see changes apply instantly. All overlay DOM elements use `panoptic-overlay-*` prefixed class names and CSS custom properties declared in `:root`, for example:

```css
--panoptic-overlay-card-background
--panoptic-overlay-album-art-width
--panoptic-overlay-track-title-text-color
```

Custom themes are provided in the [`examples/now-playing/`](examples/now-playing/) directory:
- [`now-playing-default.css`](examples/now-playing/now-playing-default.css) - Standard modern card layout.
- [`spinning-vinyl.css`](examples/now-playing/spinning-vinyl.css) - Premium circular disc style with spinning animation (customisable clockwise or widdershins) and static outer progress bar ring.

### OBS Integration

**Text file source (simplest):**
1. Add a **Text (GDI+)** source in OBS.
2. Check "Read from file" and point it to `~/.config/panoptic/current_track.txt`.
3. The text updates automatically every second.

**Browser source (full overlay):**
1. Point a Browser Source at the overlay URL served by Panoptic's local server.
2. Style with your custom CSS from the Live Overlay editor.

## Configuration

Settings are stored in your platform's standard config directory:

| Platform | Path |
|----------|------|
| **Linux** | `~/.config/com.jaintp.panoptic/settings.json` |
| **Windows** | `%APPDATA%\com.jaintp.panoptic\settings.json` |

The JSON file contains:

```json
{
  "client_id": "your-spotify-client-id",
  "access_token": "...",
  "refresh_token": "...",
  "template": "Now Playing: {title} by {artist}"
}
```

> **Security note:** Tokens are stored locally on your machine. No secrets are sent to any third party. Authentication uses PKCE, meaning no client secret exists.

## HTTP API Reference

Panoptic runs a local Axum server on **`http://127.0.0.1:3000`**:

| Endpoint | Method | Response | Description |
|----------|--------|----------|-------------|
| `/current-track` | `GET` | `text/plain` | Formatted track string from the output template |
| `/health` | `GET` | `200 OK` | Server health check |
| `/callback` | `GET` | Redirect | Spotify OAuth PKCE redirect handler (internal) |

## Development

### Running Tests

```bash
# Rust backend tests (all crates)
cargo test --workspace

# Frontend component tests
cd crates/ui/panoptic-gui
npm run test
```

### Code Quality

```bash
# Format check
cargo fmt --check

# Lint
cargo clippy -- -D warnings

# Frontend
cd crates/ui/panoptic-gui
npx tsc --noEmit
```

### CI/CD

- **Release workflow** ([`.github/workflows/release.yml`](.github/workflows/release.yml)) - triggered on `v*` tag push. Builds Linux and Windows bundles in parallel and creates a draft GitHub Release with all artifacts attached.

To cut a release:

```bash
git tag v0.2.0
git push origin v0.2.0
```

## Troubleshooting

### Wayland / Hyprland - blank window or crash

Panoptic automatically sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` on Linux to work around a WebKitGTK DMA-BUF rendering bug on Wayland compositors. If you still see issues, ensure your `webkit2gtk` package is up to date.

### Spotify shows "Not linked" despite authorising

- Check that no firewall is blocking `127.0.0.1:3000`.
- Ensure only one instance of Panoptic is running (the callback server binds exclusively to port 3000).
- Try unlinking and re-linking from the Auth tab.

### Progress bar stalling

If the progress bar freezes, the native media player may not support position queries. Panoptic defaults to `0` gracefully. The Spotify Web API fallback provides accurate progress data.

## License

See [LICENSE](LICENSE) for details.
