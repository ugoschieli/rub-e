# Asset Software

This project is a desktop application built with **Tauri v2** and **Next.js**.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) and Cargo.
- [Node.js](https://nodejs.org/) (v18+ recommended).
- Tauri CLI installed:
  ```bash
  cargo install tauri-cli
  ```

## Installation

Install the frontend dependencies:

```bash
npm install
# or
pnpm install
```

## Development

To launch the application in development mode (with hot-reload for both frontend and backend):

```bash
cargo tauri dev
```

## Build

To generate the production executable:

```bash
cargo tauri build
```
The generated files will be located in `src-tauri/target/release/bundle`.

## Tests

### Frontend (Next.js / Vitest)
To run the frontend unit tests with Vitest:

```bash
npm run test
```

### Backend (Rust)
To run the Rust tests for the Tauri application:

```bash
cd src-tauri
cargo test
```
