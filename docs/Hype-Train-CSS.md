# Twitch Hype Train Overlay CSS Guide

The **Twitch Hype Train** overlay tracks real-time progress tiers and contributors. It is fully themeable using the following structural classes and CSS variables.

## Structural Classes

Target these classes to customize the layout:

| Class Name | Description |
|------------|-------------|
| `.hype-train-card` | The main card containing the Hype Train status. |
| `.hype-idle-state` | Shown when no Hype Train is active. |
| `.hype-active-state` | Shown during an active Hype Train. |
| `.status-icon` | The icon shown when the overlay is idle (e.g., 💤). |
| `.hype-title` | The main title when idle. |
| `.hype-sub` | The subtitle when idle. |
| `.hype-active-title` | The main title during a Hype Train. |
| `.hype-level-badge` | Displays the current Hype Level (e.g., Level I). |
| `.hype-progress-track` | The background track of the progress bar. |
| `.hype-progress-fill` | The glowing fill of the progress bar. |
| `.hype-leaderboard-list` | Container for the top contributors list. |
| `.hype-leaderboard-item` | A single contributor entry in the list. |
| `.corner` | Ornaments located in the card corners (`tl`, `tr`, `bl`, `br`). |

## CSS Variables

These variables are used in the default theme and can be overridden in `:root`:

### Colors & Background
*   `--ht-color-bg`: The background color of the main card.
*   `--ht-color-primary`: The primary accent color (used for level badges and borders).
*   `--ht-color-secondary`: The secondary accent color (used for titles and highlights).
*   `--ht-color-text`: The main text color.
*   `--ht-color-subtext`: Color for labels and secondary info.
*   `--ht-color-progress`: The color of the progress bar fill.
*   `--ht-color-progress-bg`: The background of the progress bar track.
*   `--ht-color-border`: The color of the card border.

### Typography
*   `--ht-font-family`: The global font family for the overlay.

## Animation Hooks

The Hype Train overlay includes several default animations you can customize:

```css
/* Customize the card "breathe" effect */
@keyframes HTcardBreathe {
  0%, 100% { transform: scale(1); box-shadow: 0 0 25px rgba(var(--ht-color-primary-rgb), 0.15); }
  50% { transform: scale(1.02); box-shadow: 0 0 40px rgba(var(--ht-color-primary-rgb), 0.28); }
}

/* Customize the idle icon "bob" effect */
@keyframes iconBob {
  0%, 100% { transform: translateY(0) rotate(-4deg); }
  50% { transform: translateY(-5px) rotate(4deg); }
}
```

## Contributor Ranks

Each contributor in the leaderboard is marked with a rank icon. You can target these specifically:

```css
.hype-rank {
  font-size: 14px;
  filter: drop-shadow(0 0 5px var(--ht-color-secondary));
}
```
