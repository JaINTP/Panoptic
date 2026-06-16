# Twitch Alerts Overlay CSS Guide

The **Twitch Alerts** overlay supports a high-fidelity, multi-alert queue with dynamic stacking and professional transitions. It can be fully customized using specific CSS classes and variables.

## Structural Classes

Target these classes to manage the alert stack and individual cards:

| Class Name | Description |
|------------|-------------|
| `#overlay-container` | The fixed container for the entire alert stack. |
| `.alert-node` | Wrapper for each individual alert in the stack (used for transitions). |
| `.alert-card` | The main styling card for a single alert. |
| `.alert-text-content` | The container for the rendered alert message. |
| `.hype-active-title` | The header label (e.g., FOLLOW, SUB). |
| `.hype-icon` | The icon representing the alert type. |
| `.fade-out` | Class added when an alert is expiring and leaving the screen. |

### Type-Specific Classes
Each alert is tagged with a type-specific class for granular control:
*   `.alert-follow`
*   `.alert-subscription`
*   `.alert-giftsubscription`
*   `.alert-raid`
*   `.alert-cheer`

## CSS Variables

Use these variables in the `#overlay-container` or `:root` to control the stack behavior:

| Variable | Description | Default |
|----------|-------------|---------|
| `--container-bottom` | Position from the bottom of the screen. | `20px` |
| `--container-right` | Position from the right of the screen. | `20px` |
| `--stack-direction` | Direction of the alert stack (`column` or `column-reverse`). | `column-reverse` |
| `--stack-gap` | Space between individual alerts in the stack. | `12px` |
| `--alert-duration` | Visual timer duration (must match settings duration). | `8s` |

## The "Settle" Transition

When an alert expires, the remaining alerts shift positions. You can customize this movement using the `.alert-node` transition:

```css
.alert-node {
  /* A bouncy transition for professional movement */
  transition: all 0.6s cubic-bezier(0.34, 1.56, 0.64, 1);
}
```

## Entrance & Exit Animations

You can define custom animations for when an alert appears and disappears:

```css
/* Entry Animation */
.alert-card.active {
  animation: mySlideIn 0.5s ease-out both;
}

@keyframes mySlideIn {
  from { opacity: 0; transform: translateX(50px); }
  to { opacity: 1; transform: translateX(0); }
}

/* Exit Transition */
.alert-node.fade-out {
  opacity: 0;
  transform: scale(0.9);
  margin-bottom: -80px; /* Collapse space so others drop down */
}
```

## Progress Bar Timer

By default, alerts include a visual progress bar that counts down their duration:

```css
.alert-card::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  height: 3px;
  background: var(--ht-color-primary);
  width: 100%;
  animation: alertTimer var(--alert-duration) linear forwards;
}

@keyframes alertTimer {
  from { width: 100%; }
  to { width: 0%; }
}
```
迫
