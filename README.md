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

The site is pure HTML, CSS, and JavaScript located in the [`docs/`](docs)
folder so it can be deployed via GitHub Pages using the “Deploy from a branch”
option targeting the `main` branch and `/docs` directory.

To preview locally run a static file server from the repository root, then open
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
