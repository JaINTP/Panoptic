# Panoptic Product Roadmap

This document outlines the planned evolution of the Panoptic toolkit. Our focus is on high-performance, modular utilities that empower streamers with deep customizability.

## 🔌 Planned Plugins

### 💬 Twitch Chat Overlay — Thematic Filtering
Extend the live chat overlay with cross-plugin visual reactivity.
- **Status:** Planning
- **Key Feature:** Thematic Filtering—chat messages trigger visual effects across other overlays (e.g. a keyword flashes the hype train, a sub triggers a global colour shift).
- **Shipped in v0.1.9:** Core chat overlay with badge/emote image resolution, pronoun display, and configurable frame and animation styles.

### 🏆 Universal Stream Goals
Dynamic progress bars for Followers, Subscribers, and Bits with advanced animation support.
- **Status:** Planning
- **Key Feature:** Multi-Stage Goals—reaching a milestone can automatically trigger a theme swap.

### 🎙️ "Talk-Back" Avatar Visualizer
A microphone-reactive indicator or character for streamers who prefer not to use a face-cam.
- **Status:** Concept
- **Key Feature:** Real-time lip-sync and movement driven by system audio amplitude.

### 🌉 Discord Rich Presence Bridge
Seamlessly sync your "Now Playing" music and current stream alerts directly to your Discord status.
- **Status:** Concept
- **Key Feature:** Show alert history and current track artwork in Discord.

### 🎮 Retro Gaming ROM Fetcher
Automatically update "Now Playing" with the metadata and box art of games running in common emulators.
- **Status:** Concept
- **Key Feature:** IGDB integration for high-quality artwork.

### ⚡ Interactive "Bit-Triggers"
Enable viewers to manipulate overlay visuals directly using Twitch Bits.
- **Status:** Concept
- **Key Feature:** "Chaos Mode"—large cheers can trigger temporary global glitch or glow effects.

### ~~⏲️ Pomodoro / BRB Timer~~
~~Themed countdown timers for focus sessions or break screens.~~
- ~~**Status:** Planning~~
- ~~**Key Feature:** Automation hooks—pause music or trigger alerts when the timer expires.~~
- **Shipped in v0.2.0:** Work/break cycle timer with configurable durations, session dots, circular progress ring overlay, phase-complete automation event (`pomodoro_phase_complete`), and full CSS variable theming.

## 🛠️ Core Engine Improvements

- **Global Theme Swapper:** A single button in the UI to instantly swap between aesthetic packs across all plugins.
- **Remote Configuration:** Secure web dashboard to tweak settings from a secondary device (phone/tablet).
- **OBS WebSocket Integration:** Allow Panoptic actions to control OBS scenes and sources directly.

---
*Note: This roadmap is subject to change based on community feedback and developer bandwidth.*
