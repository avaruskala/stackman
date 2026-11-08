# Stackman

Stackman is a web application for inspecting, testing, and editing Moodle XML question banks. It parses STACK questions, multichoice questions, and true or false questions. It renders CASText and mathematical formulas with MathJax, and it runs Maxima to evaluate question variables, inputs, and potential response trees.

## Prerequisites

1. Rust toolchain with the WebAssembly target:
```bash
rustup target add wasm32-unknown-unknown
```
2. cargo-leptos:
```bash
cargo install cargo-leptos --locked
```
3. Maxima installed at `/usr/bin/maxima` on Linux. The server runs Maxima in a separate subprocess to evaluate question variables and generate question variants.
4. Sass compiler. You can install `dart-sass` with npm:
```bash
npm install -g sass
```

## Running the application

To run Stackman in development mode with automatic rebuilds:
```bash
cargo leptos watch
```
The server listens on `http://127.0.0.1:3000`.

## Building for release

To compile the release binaries and the client site bundle:
```bash
cargo leptos build --release
```
The build produces the server executable in `target/server/release/server` and client assets in `target/site`.

## Testing

Run unit tests for all workspace crates:
```bash
cargo test --workspace
```
To run tests only for the XML parser and CAS evaluation crate:
```bash
cargo test -p stack
```

## Workspace structure

The workspace divides responsibilities into four crates:

* `crates/stack`. Contains Moodle XML parsers, STACK data structures, Maxima script generation, CASText rendering, and STACK-JS block parsers. It does not depend on web frameworks or async runtimes.
* `crates/app`. Contains the Leptos user interface, state management, question filtering, toolbar controls, preview cards, and client scripts.
* `crates/server`. Contains the Axum HTTP server. It serves Leptos server routes, static files from `public/`, and sets CORS headers on `/corsscripts/`.
* `crates/frontend`. Contains the WebAssembly entry point that mounts the client application in the browser.

## Capabilities

* **Moodle XML support.** The parser accepts `<quiz>`, individual `<question>`, and `<question type="stack">` formats, and preserves metadata and embedded base64 files.
* **Deterministic random seeds.** The Maxima runner initializes the random state with an explicit integer seed so repeated runs produce identical variable values, answers, and plots.
* **Static fallback.** When Maxima is absent or a CAS command fails, the renderer returns a static preview with input slots and unrendered formulas instead of stopping execution.
* **STACK-JS block isolation.** The renderer places `[[iframe]]`, `[[javascript]]`, and `[[jsxgraph]]` scripts into sandboxed iframes and routes input updates through postMessage events.
* **Answer inspection.** Users can fill inputs with teacher answers, check matrix dimensions, and inspect potential response tree branches.
