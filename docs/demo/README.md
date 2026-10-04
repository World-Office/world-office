# World-Office Document Editor Demo

This directory contains a visual demonstration of the World-Office document editor UI.

## Screenshots

| File | Description |
|------|-------------|
| `screenshots/editor-ui.png` | Full editor interface - main canvas, toolbar, and status areas |
| `screenshots/toolbar.png` | Toolbar region showing editing controls and action buttons |
| `screenshots/collaboration-status.png` | Collaboration status widget showing connected users |
| `screenshots/remote-cursor-demo.png` | Document canvas area with remote cursor overlays |

## Demo Video

**File:** `demo.mp4` (30 seconds)

### Chapter Markers

| Time | Segment |
|------|---------|
| 0:00 - 0:05 | Editor loads, initial render |
| 0:05 - 0:10 | Editor UI fully visible |
| 0:10 - 0:20 | Document rendering complete |
| 0:20 - 0:30 | Final view, editor idle |

## Features Demonstrated

- **Document Canvas**: Multi-page document rendering with zoom controls
- **Toolbar**: Access to common editing operations (new, open, save, export)
- **Page Navigation**: Previous/Next page navigation controls
- **Collaboration Integration**: WebSocket-based real-time collaboration connectivity
- **Status Indicators**: Document state and collaboration status display

## Technical Details

- **Browser**: Playwright chromium (headless)
- **Video Capture**: ffmpeg with x11grab, H.264 encoding
- **Editor Dev Server**: Vite + React, port 3006

## How to View

Open the PNG files in any image viewer. The video can be played with any media player supporting H.264/MP4 (VLC, mpv, ffplay, etc.).