# Now Playing Overlay CSS Guide

The **Now Playing** overlay is highly customizable via CSS. It uses a robust set of CSS variables and structural classes to give you total control over the layout and aesthetic.

## Structural Classes

Use these classes in your CSS to target specific elements:

| Class Name | Description |
|------------|-------------|
| `.panoptic-overlay-wrapper` | The top-level container for the overlay. |
| `.panoptic-overlay-card` | The main display card containing all info. |
| `.panoptic-overlay-art-container` | Wrapper for the album artwork. |
| `.panoptic-overlay-album-art` | The `<img>` element for the artwork. |
| `.panoptic-overlay-text-container` | Container for track, artist, and album info. |
| `.panoptic-overlay-track-title` | The song title text. |
| `.panoptic-overlay-track-artist` | The artist name text. |
| `.panoptic-overlay-track-album` | The album name text. |
| `.panoptic-overlay-progress-section` | Container for the progress bar and time. |
| `.panoptic-overlay-progress-container` | The background/track of the progress bar. |
| `.panoptic-overlay-progress-bar` | The actual fill of the progress bar. |
| `.panoptic-overlay-time-display` | Container for current time and duration. |
| `.panoptic-overlay-time-current` | The current playback time text. |
| `.panoptic-overlay-time-separator` | The `/` character between times. |
| `.panoptic-overlay-time-duration` | The total song duration text. |

## CSS Variables

Configure these in the `:root` selector or within `.panoptic-overlay-card` for easy global adjustments.

### Card Layout & Background
*   `--panoptic-overlay-card-display`: Controls the layout mode (usually `flex`).
*   `--panoptic-overlay-card-gap`: The space between the album art and text.
*   `--panoptic-overlay-card-background`: Background color or image for the card.
*   `--panoptic-overlay-card-border-style`: Border style (e.g., `solid`, `dashed`).
*   `--panoptic-overlay-card-border-width`: Thickness of the card border.
*   `--panoptic-overlay-card-border-color`: Color of the card border.
*   `--panoptic-overlay-card-backdrop-blur`: Blurs everything behind the card (e.g., `16px`).
*   `--panoptic-overlay-card-padding`: Inner spacing of the card.
*   `--panoptic-overlay-card-border-radius`: Roundness of the card corners.
*   `--panoptic-overlay-card-width`: Total width of the card.
*   `--panoptic-overlay-card-box-shadow`: Outer and inner shadow effects.

### Album Art
*   `--panoptic-overlay-album-art-width`: Width of the artwork image.
*   `--panoptic-overlay-album-art-height`: Height of the artwork image.
*   `--panoptic-overlay-album-art-border-radius`: Roundness of the art corners.
*   `--panoptic-overlay-album-art-object-fit`: How the image fits (`cover` or `contain`).
*   `--panoptic-overlay-album-art-box-shadow`: Shadow effect for the artwork.

### Typography
*   `--panoptic-overlay-track-title-font-family`: Font for the song title.
*   `--panoptic-overlay-track-title-font-size`: Size of the song title.
*   `--panoptic-overlay-track-title-font-weight`: Thickness of the title text.
*   `--panoptic-overlay-track-title-text-color`: Color of the song title.
*   `--panoptic-overlay-track-title-text-shadow`: Glow or shadow for the title.
*   `--panoptic-overlay-track-artist-font-size`: Size of the artist name.
*   `--panoptic-overlay-track-artist-font-weight`: Thickness of the artist text.
*   `--panoptic-overlay-track-artist-text-color`: Color of the artist name.
*   `--panoptic-overlay-track-album-font-size`: Size of the album name.
*   `--panoptic-overlay-track-album-text-color`: Color of the album name.
*   `--panoptic-overlay-track-album-letter-spacing`: Tracking for the album text.

### Progress Bar & Time
*   `--panoptic-overlay-progress-bar-height`: Height of the progress bar fill.
*   `--panoptic-overlay-progress-bar-background`: Background color of the bar track.
*   `--panoptic-overlay-progress-bar-border-radius`: Roundness of the bar.
*   `--panoptic-overlay-progress-bar-fill-gradient`: Color or gradient of the fill.
*   `--panoptic-overlay-time-display-font-family`: Font for the time text.
*   `--panoptic-overlay-time-display-font-size`: Size of the time text.
*   `--panoptic-overlay-time-display-text-color`: Color of the time text.

## State Selectors

You can apply styles based on the current playback state:

```css
/* Styling for when music is paused */
.panoptic-overlay-wrapper[data-playing="false"] .panoptic-overlay-card {
  opacity: 0.8;
  filter: grayscale(0.5);
}
```
