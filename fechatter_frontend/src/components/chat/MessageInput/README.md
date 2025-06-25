# MessageInput Component

This is the enhanced MessageInput component with a modular folder structure.

## Structure

```
MessageInput/
├── index.vue       # Main component file
├── styles.css      # Component styles (separated for better organization)
└── README.md       # This documentation
```

## Features

### Core Functionality
- ✅ **Smart Layout**: Preview → Toolbar → Input (correct order)
- ✅ **Enhanced Markdown Toolbar**: Complete set of formatting buttons
- ✅ **Dynamic Send Button**: Visual states (empty → filled → pulse)
- ✅ **Multi-format Support**: Text, Markdown, Code, File modes
- ✅ **File Upload**: Drag & drop, paste support


### Markdown Toolbar Sections

#### Text Formatting
- Bold (**bold**) - Ctrl+B
- Italic (*italic*) - Ctrl+I  
- Strikethrough (~~text~~)
- Highlight (==text==)
- Inline Code (`code`)

#### Structure
- H1, H2, H3 headings
- Unordered lists (-)
- Ordered lists (1.)
- Task lists (- [ ])

#### Content
- Quotes (>)
- Code blocks (```)
- Links - Ctrl+K
- Images
- Tables
- Horizontal rules

#### Actions
- Preview toggle
- Clear content

### Send Button States
- **Empty**: Subdued, dashed border (discourages sending)
- **Filled**: Gradient, shadow (encourages sending)
- **Pulse**: Animation for longer messages (5+ chars)

## Usage

```vue
<template>
  <MessageInput 
    :chat-id="chatId"
    :reply-to-message="replyMessage"
    @message-sent="handleMessageSent"
    @preview-state-change="handlePreviewChange"
  />
</template>

<script>
import MessageInput from '@/components/chat/MessageInput';

export default {
  components: {
    MessageInput
  }
}
</script>
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `chatId` | Number/String | required | Current chat ID |
| `replyToMessage` | Object | null | Message to reply to |
| `disabled` | Boolean | false | Disable input |
| `maxLength` | Number | 2000 | Max character limit |

## Events

| Event | Payload | Description |
|-------|---------|-------------|
| `message-sent` | MessageData | Emitted when message is sent |
| `reply-cancelled` | - | Emitted when reply is cancelled |
| `preview-state-change` | Boolean | Emitted when preview state changes |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Enter | Send message |
| Shift+Enter | New line |
| Ctrl/Cmd+B | Bold (Markdown mode) |
| Ctrl/Cmd+I | Italic (Markdown mode) |
| Ctrl/Cmd+K | Link (Markdown mode) |

## Styling

The component uses CSS variables for theming:
- `--color-primary`: Primary brand color
- `--color-background`: Background colors
- `--color-text`: Text colors
- `--color-border`: Border colors

## Future Enhancements

### Planned Sub-components
- `MarkdownToolbar.vue`: Dedicated toolbar component
- `FilePreview.vue`: File upload preview
- `composables/`: Reusable logic hooks

### Composables
- `useMessageInput.js`: Core input logic
- `useFileHandling.js`: File upload logic
- `useMarkdownEditor.js`: Markdown editing logic
