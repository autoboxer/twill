# Twill

Knowledge, woven to last.

Twill is a private, offline-first study app built with Tauri 2 and Vue 3. This repository currently contains the minimal application foundation for desktop and mobile development.

## Prerequisites

- A current Node.js LTS release
- The stable Rust toolchain
- The [Tauri system dependencies](https://v2.tauri.app/start/prerequisites/) for your platform

Android development also requires Android Studio and the Android SDK. iPhone and iPad development requires macOS and Xcode.

## Development

Install the JavaScript dependencies:

```sh
npm install
```

Run Twill as a desktop application:

```sh
npm run tauri dev
```

Run only the browser-based frontend:

```sh
npm run dev
```

## Checks

Run the JavaScript linter, frontend build, and Rust compiler checks:

```sh
npm run check
```
