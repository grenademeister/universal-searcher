## Project Architecture

- **Tauri app (single binary)**  
  - Backend logic lives in `src-tauri/src/overlay.rs` and is exposed via the `generate_overlay` command in `src-tauri/src/lib.rs`.  
  - Reads Wayland selection/clipboard via `wl-paste`, then calls OpenAI or Gemini based on the requested provider and returns plain text to the frontend.  
  - Frontend (`src/App.jsx` + `src/App.css`) renders the overlay text; press `Tab` to switch to Gemini. Window config sits in `src-tauri/tauri.conf.json` (480x300, frameless, transparent, always-on-top, resizable).

## Configuration

- Environment variables:  
  - `OPENAI_API_KEY` (required for OpenAI), optional `OPENAI_MODEL` (default `gpt-4o-mini`).  
  - `GEMINI_API_KEY` or `GEMINI_API_TOKEN` (required for Gemini), optional `GEMINI_MODEL` (default `gemini-2.5-flash`).  
  - `OVERLAY_PROMPT` overrides the default concise system prompt.
- Clipboard dependency: requires `wl-paste`; the backend tries primary selection first, then clipboard.

## Build & Run

- Development: `npm run tauri dev` (runs `npm run dev` for the frontend).  
- Production: `npm run tauri build` (runs `npm run build` and bundles a single Tauri binary; no extra CLI artifact needed).
