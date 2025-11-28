## Project Architecture (Current State)

- **Rust CLI (`overlay-cli`)**  
  - Location: `src-tauri/src/bin/overlay_cli.rs`  
  - Responsibility: Reads Wayland selection/clipboard via `wl-paste`, calls OpenAI Chat Completions, prints plain text to stdout. Uses env vars `OPENAI_API_KEY`, optional `OPENAI_MODEL` (default `gpt-4o-mini`), and optional `OVERLAY_PROMPT`.

- **Tauri App (Shell/Overlay)**  
  - Rust entry: `src-tauri/src/lib.rs` (exposes `run_overlay_cli` command that spawns the CLI; auto-detects bundled CLI in resources, or uses `OVERLAY_CLI_PATH`, or PATH).  
  - Frontend: `src/App.jsx` + `src/App.css` — minimal UI that only renders the CLI output (or loading/error text). Window config in `src-tauri/tauri.conf.json` (currently 480x300, frameless, transparent, always-on-top, resizable).
  - Build packaging: `npm run tauri build` runs `npm run build` + `npm run build:overlay-cli`, bundles `resources/overlay-cli`, and ships both binaries together.

- **Scripts & Resources**  
  - `package.json`: dev/build scripts; `build:overlay-cli` builds the CLI and copies it to `src-tauri/resources/overlay-cli`.  
  - `src-tauri/capabilities/default.json`: permissions including window set-size/center for the overlay window.

## Where API Calls Happen

- OpenAI chat call is implemented in `src-tauri/src/bin/overlay_cli.rs` inside `query_openai`, using `reqwest` with rustls.  
  - Inputs: `OPENAI_API_KEY`, `OPENAI_MODEL` (default `gpt-4o-mini`), `OVERLAY_PROMPT` (default concise prompt), selected text from `wl-paste`.  
  - Output: stdout plaintext for the Tauri frontend to display.
