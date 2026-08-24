# Twill

Twill is a private, local-first study application for concept-based retrieval practice and spaced repetition. It stores the complete library and review history on the learner's device, uses FSRS for scheduling, and does not require an account or hosted server.

The application is built with Tauri 2, Vue 3, JavaScript, and Rust. Linux is the current desktop development target; Android, iPhone, and iPad support is planned but not integrated yet.

## Prerequisites

- A current Node.js LTS release
- The stable Rust toolchain
- The [Tauri system dependencies](https://v2.tauri.app/start/prerequisites/) for the development platform

## Install

Clone the repository and install the JavaScript dependencies:

```sh
git clone https://github.com/autoboxer/twill.git
cd twill
npm install
```

## Run

Launch the native desktop application in development mode:

```sh
npm run tauri dev
```

Tauri starts the Vite development server automatically and opens the application window.

To run only the browser-based frontend:

```sh
npm run dev
```

Browser mode is useful for frontend work, but native persistence and other Tauri commands are unavailable outside the desktop application.

## Customize appearance

Themes, fonts, and motion preferences are available in Settings. For
validated local CSS overrides, see the [CSS snippet guide](docs/css-snippets.md).

## Test and check

Run the Rust test suite:

```sh
npm test
```

Run the JavaScript linter, frontend production build, and Rust compiler checks:

```sh
npm run check
```

## Build

Build the native application and packages supported by the current platform:

```sh
npm run tauri build
```

Build only the frontend assets:

```sh
npm run build
```
