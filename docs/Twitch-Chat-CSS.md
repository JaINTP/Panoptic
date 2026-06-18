# Twitch Chat Overlay CSS Guide

The **Twitch Chat** overlay provides a high-performance, real-time message stream with integrated user pronouns and role-based styling. It is fully themeable and uses the same structural framework as other Panoptic overlays.

## Structural Classes

Target these classes to customize your chat look:

| Class Name | Description |
|------------|-------------|
| `#chat-container` | The fixed container for the entire message list. |
| `.chat-message` | The container for a single chat message (inherits `.panoptic-overlay-card`). |
| `.chat-header` | The top section of a message containing name and pronouns. |
| `.chat-pronouns` | The element displaying user pronouns (e.g., [he/him]). |
| `.chat-username` | The chatter's display name. |
| `.chat-text` | The actual message content. |
| `.chat-badges-wrap` | Container for user role badges. |

### Role-Based Classes
Each message is tagged with the sender's role for specific styling:
*   `.chat-message-broadcaster`
*   `.chat-message-mod`
*   `.chat-message-vip`
*   `.chat-message-sub`

## CSS Variables

Use these variables in your stylesheet for global control:

| Variable | Description | Default |
|----------|-------------|---------|
| `--chat-container-width` | The width of the chat stack. | `400px` |
| `--chat-stack-direction` | Direction of messages (`column` or `column-reverse`). | `column` |
| `--chat-message-gap` | Space between messages. | `8px` |

## Pronouns & Badges

Panoptic automatically fetches user pronouns from **pronouns.alejo.io**. You can style them specifically:

```css
.chat-pronouns {
  font-size: 0.8em;
  opacity: 0.6;
  margin-right: 6px;
  font-weight: 700;
  color: var(--ht-color-primary);
}

.chat-badges-wrap span {
  font-size: 0.7em;
  padding: 2px 4px;
  background: rgba(255,255,255,0.1);
  margin-right: 4px;
  border-radius: 3px;
}
```

## Animation Hooks

Messages enter the stack using the `chatSlideIn` animation by default. You can customize this in your theme:

```css
@keyframes chatSlideIn {
  from { 
    opacity: 0; 
    transform: translateX(-30px) scale(0.9);
    filter: blur(5px);
  }
  to { 
    opacity: 1; 
    transform: translateX(0) scale(1);
    filter: blur(0);
  }
}
```

## Configuration

In the **Display** tab under **Twitch Chat**, you can configure:
*   **Message Template:** Define the order of elements using `{user}`, `{message}`, `{pronouns}`, and `{badges}`.
*   **Show Pronouns:** Toggle pronouns visibility globally.
*   **Show Badges:** Toggle role badges visibility.
*   **Max Messages:** Set how many messages are kept on screen before the oldest are removed.
*   **Test Message:** Use the "Simulate Message" button to see your styling in action.
迫
