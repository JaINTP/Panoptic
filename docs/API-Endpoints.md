# Panoptic API Endpoints

Panoptic runs a local Axum HTTP server at `http://localhost:3000` to serve overlays to OBS, expose live state to widgets, and handle programmatic updates to variables.

---

## 1. Core Endpoints

### `GET /health`
- **Description:** Basic health check to verify the local server is running.
- **Response:** `OK` (plain text, 200 OK)

### `GET /current-track`
- **Description:** Returns the formatted now-playing track string.
- **Response:** Plain text representation (e.g. `Song - Artist`)

### `GET /playback`
- **Description:** Returns the full current media playback state.
- **Response:** JSON payload:
  ```json
  {
    "title": "Song Title",
    "artist": "Artist Name",
    "album": "Album Name",
    "is_playing": true,
    "progress_ms": 45000,
    "duration_ms": 180000,
    "art_url": "http://localhost:3000/art?v=...",
    "provider": "spotify"
  }
  ```

### `GET /art`
- **Description:** Serves the album art image for the current track. Supports caching/invalidation via query parameter.
- **Query Parameters:**
  - `v` (optional): Unique hash value of the track art to bypass browser cache.
- **Response:** Binary image file (JPEG/PNG)

### `GET /callback/:provider`
- **Description:** Redirect callback handler for OAuth authentication (e.g., Spotify/Twitch).

---

## 2. Overlay Pages (OBS Browser Sources)

These endpoints serve HTML/JS widgets styled via settings. Add them as **Browser Sources** in OBS.

- **Now Playing Overlay:** `GET /overlay/now-playing`
- **Pomodoro Timer:** `GET /overlay/pomodoro`
- **Stream Goals:** `GET /overlay/stream-goals`
- **Twitch Alert Stack:** `GET /overlay/twitch/alerts`
- **Twitch Live Chat:** `GET /overlay/twitch/chat`
- **Twitch Hype Train:** `GET /overlay/twitch/hype-train`

---

## 3. Plugin State / Data Endpoints

These endpoints return the active data/configuration for specific plugins as JSON.

### `GET /pomodoro/state`
- **Description:** Returns active Pomodoro timer state.
- **Response:** JSON representation of phase, durations, and session counts.

### `GET /stream-goals/state`
- **Description:** Returns active stream goals configuration, current Twitch event variables, and custom variables.
- **Response:** JSON payload containing goals and all resolved variables.

### `GET /twitch/alerts`
- **Description:** Returns active stacking Twitch alerts and alert configuration.
- **Response:** JSON payload of the active alerts stack.

### `GET /twitch/chat`
- **Description:** Returns the list of recent resolved chat messages and chat layout settings.
- **Response:** JSON payload of message history.

### `GET /twitch/hype-train`
- **Description:** Returns the current level, point progression, and top contributor list for the Twitch Hype Train.
- **Response:** JSON payload of the active Hype Train status.

---

## 4. Custom Variable Mutators (Stream Goals)

These endpoints allow external software (like OBS, Streamer.bot, or custom scripts) to programmatically update custom stream goal counters. 

All mutators save updates directly to `settings.json`, notify the frontend UI in real-time, and return a JSON response with the updated value.

### `POST /stream-goals/variable/:name/increment`
- **Description:** Increments the specified custom variable's value by its defined step amount.
- **Path Parameter:** `name` - The exact name of the custom variable (e.g., `deaths`).
- **Response (200 OK):**
  ```json
  {
    "name": "deaths",
    "value": 11.0
  }
  ```

### `POST /stream-goals/variable/:name/decrement`
- **Description:** Decrements the specified custom variable's value by its defined step amount.
- **Path Parameter:** `name` - The exact name of the custom variable.
- **Response (200 OK):**
  ```json
  {
    "name": "deaths",
    "value": 10.0
  }
  ```

### `POST /stream-goals/variable/:name/set`
- **Description:** Sets the specified custom variable to an absolute target value.
- **Path Parameter:** `name` - The exact name of the custom variable.
- **Query Parameter or JSON Body:** Requires `value` parameter as a float.
  - *Via Query:* `POST /stream-goals/variable/deaths/set?value=15`
  - *Via JSON Body:* `POST /stream-goals/variable/deaths/set` with payload `{"value": 15}`
- **Response (200 OK):**
  ```json
  {
    "name": "deaths",
    "value": 15.0
  }
  ```
