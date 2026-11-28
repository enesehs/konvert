# Contributing to Konvert

Thank you for your interest in contributing to **Konvert**. Please follow these guidelines to keep the codebase safe, fast, and maintainable.

## Development Setup

- **Rust toolchain:** Rust 2024 edition (`cargo`, `rustfmt`, `clippy`)
- **Qt6 GUI:** `cmake`, `ninja`, `qt6-base` (Widgets, Core, Gui, Network), `qt6-svg`
- **Runtime delegates (for testing):** `ffmpeg`, `imagemagick`, `ghostscript`

## Code & Safety Rules

1. **No panics in production Rust code:** Do not use `.unwrap()` or `.expect()` outside tests. Propagate errors cleanly with `Result` and `?`.
2. **No shell execution:** Never invoke external tools through `sh -c` or string-interpolated shell commands. Always use direct `std::process::Command` argument arrays and sanitize file paths.
3. **Atomic file I/O:** Write outputs to temporary `.part` files and commit atomically, respecting collision avoidance and `--backup` semantics.
4. **Offline & zero telemetry:** Konvert must remain 100% offline. Network requests are only allowed in explicit update-check commands (`--check-update`).
5. **Cross-distro Qt6 compatibility:** Keep `CMAKE_POSITION_INDEPENDENT_CODE ON` in `gui/CMakeLists.txt` and avoid relying on unreleased Qt6 APIs.
6. **Installer parity:** If you modify `scripts/install.sh`, keep `docs/public/install.sh` in sync and regenerate `docs/public/install.sh.sha256`.
7. **No emojis:** Keep source code, CLI output, GUI strings, and documentation free of emojis.

## Required Checks Before Opening a PR

Run all checks locally and ensure they pass with zero warnings:

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

If you modified the Qt6 Settings GUI (`gui/`):

```bash
cmake -B gui/build -S gui -DCMAKE_BUILD_TYPE=Release
cmake --build gui/build
```

## Pull Request Workflow

1. Fork the repository and create a branch from `main` (e.g., `feat/my-feature` or `fix/issue-name`).
2. Follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`).
3. Check [`todo.md`](todo.md) for planned `v0.6.0` items if you are looking for something to work on.
4. Open a Pull Request describing what changed and how it was verified.
