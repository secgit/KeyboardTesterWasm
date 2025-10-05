# Keyboard Repeat Pattern Explorer

A lightweight web app for observing how your system generates repeated
`keydown` events when multiple keys are held simultaneously. Use it to compare
repeat behavior across keyboards, browsers, and operating systems.

## Live demo

Once this repository is published to GitHub Pages the site will be available at:

```
https://<your-github-username>.github.io/KeyboardTesterWasm/
```

(Replace `<your-github-username>` with the account or organization that owns the
repository.)

## Local development

The interface lives in the [`docs/`](docs) folder so it can be published with
GitHub Pages using the “Deploy from a branch” option targeting the `main` branch
and `/docs` directory. All interaction logic now runs inside a Rust WebAssembly
module that is embedded into the page as base64 text so no binary artifacts need
to be committed.

### Build the WebAssembly bundle

Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/) if you do
not already have it:

```bash
cargo install wasm-pack
```

Compile the Rust crate under [`wasm/`](wasm) to WebAssembly outputs inside
`docs/pkg`:

```bash
wasm-pack build wasm --target web --out-dir docs/pkg --release
```

Then convert the generated `.wasm` file into the embedded base64 module and
remove the binary artifact:

```bash
./scripts/embed-wasm.sh
```

> **Note:** `wasm-pack` is configured to skip `wasm-opt` so the build works in
> minimal environments. If you have Binaryen installed you can remove that
> override from [`wasm/Cargo.toml`](wasm/Cargo.toml) to enable additional
> optimizations.

### Preview the site locally

Serve the repository root with any static file server and open
`http://localhost:8000/docs` in a browser:

```bash
python3 -m http.server 8000
```

## How it works

* **Held keys panel** – shows which keys are currently pressed.
* **Repeat pattern** – displays the order of recent repeated `keydown` events
  plus a running tally per key.
* **Event timeline** – logs every `keydown` and `keyup` event with timestamps and
  inter-event deltas to help you detect rotation patterns or biases.

Use the **Pause capture** toggle to temporarily stop logging without resetting
existing data, or **Clear log** to reset the session.
