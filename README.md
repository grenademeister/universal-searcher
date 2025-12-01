# Search Overlay

A Wayland desktop overlay that answers questions about selected text using AI (OpenAI, Gemini) or offline Wikipedia (Kiwix).

## How It Works

1. Reads selected text from Wayland clipboard (`wl-paste`)
2. Sends it to the chosen provider (Wikipedia, Gemini, or OpenAI)
3. Displays the response in a frameless, always-on-top overlay

**Keyboard controls:**
- `Tab` — cycle providers
- `↑/↓` — switch models within provider
- `Space` — close

## Requirements

- Linux with Wayland
- `wl-paste` installed
- For AI providers: `OPENAI_API_KEY` or `GEMINI_API_KEY`
- For Wikipedia: local Kiwix server at `localhost:8080`

## Configuration

Create `~/.config/search/search.config` (TOML) to override the overlay appearance and window size:

```toml
[appearance]
# CSS color values
background_color = "#0b1220"
font_color = "#e9ecf1"

[window]
# Dimensions in physical pixels
width = 640
height = 400
```

Missing or invalid values fall back to the defaults (transparent background, light text, 480x300 window).

## Run

```bash
# Development
npm install
npm run tauri dev

# Production build
npm run tauri build
```
