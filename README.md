# Nexus Workspace

This repository contains the **nexus** Cargo workspace. It hosts shared libraries alongside a Bevy desktop application that can be launched locally.

## Workspace layout

- `crates/anchor` – placeholder logic for configuring and initializing anchor services.
- `crates/http_api` – simple Axum router exposing a health endpoint.
- `crates/world_state` – world entity tracking primitives built with `glam` vectors.
- `apps/app` – Bevy application that opens an empty window with 2D and 3D cameras.

## Prerequisites

- Rust toolchain (Rust 1.75+ is recommended). Install via [`rustup`](https://rustup.rs/).

## Building

```sh
cargo build
```

This command compiles all workspace members, including the Bevy application and libraries.

## Running the app

```sh
cargo run -p app
```

Running the binary opens an empty Bevy window with a custom clear color and both 2D and 3D cameras. Close the window to stop the program.
