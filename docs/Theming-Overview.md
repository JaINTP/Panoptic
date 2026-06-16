# Panoptic Theming Overview

Panoptic provides a powerful, CSS-driven engine for creating professional-grade stream overlays. Every component is designed to be highly customizable while remaining structurally sound.

## How Theming Works

1.  **Physical CSS Files:** Every overlay's CSS is stored in `~/.config/panoptic/overlays/`. You can edit these files manually or via the Panoptic UI.
2.  **Atomic Versioning:** When you save your CSS, the Axum server increments a global `CSS_VERSION`. OBS browser sources poll for this version and automatically re-inject the new styles without reloading the page.
3.  **Side-by-Side Editor:** The **Display** tab provides a real-time preview of your overlays alongside a professional CSS editor (powered by CodeMirror) with autocompletion for Panoptic-specific classes.

## General Theming Tips

*   **Universal Colors:** Use CSS variables in the `:root` selector to maintain consistency across different overlays.
*   **Sticky Previews:** The Panoptic UI uses stationary previews so you can always see your visual changes, even while scrolling through long text configuration forms.
*   **Single File Themes:** Master themes (like `cyber_complete.css`) can be pasted into any overlay editor. They contain logic to style Now Playing, Hype Trains, and Alerts simultaneously.

## Component-Specific Guides

For detailed class lists and variable explanations, see the individual component guides:

*   [**Now Playing Overlay**](Now-Playing-CSS.md)
*   [**Twitch Hype Train**](Hype-Train-CSS.md)
*   [**Twitch Alerts**](Twitch-Alerts-CSS.md)

## Master Theme Library

Panoptic ships with three high-fidelity aesthetic packs located in `examples/themes/`:

*   **Cyber-Neon:** Futuristic high-contrast pink/cyan with glitch effects.
*   **Eldritch Horror:** Dark void purples with organic "breathing" animations.
*   **1990s RPG:** Classic 16-bit console UI with Royal Blue window skins.
