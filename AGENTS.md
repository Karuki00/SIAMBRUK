# Repository Guidelines

## Project Structure & Module Organization

The application lives in `tauri-app/`; the parent directory contains project specifications and contributor documentation. Keep the project layout predictable:

- `tauri-app/src/` — Vue application source, organized by feature or domain.
- `tauri-app/src/components/` — reusable UI components, including accessible controls for the Three.js inventory environment.
- `tauri-app/src-tauri/` — Rust backend commands, Tauri configuration, capabilities, and icons.
- `tauri-app/src-tauri/migrations/` — ordered SQLite schema migrations.
- `tauri-app/tests/` — automated tests, mirroring relevant source paths as they are added.
- `tauri-app/public/` — static frontend assets.
- `DESIGN_RULES.md` and `MASTER_PROJECT.md` — design and product specifications.

Keep generated output and local dependencies (for example, `dist/`, `build/`, and `node_modules/`) out of version control.

## Build, Test, and Development Commands

Run application commands from `tauri-app/`. The initialized project currently provides:

```text
npm.cmd install       # install frontend dependencies
npm.cmd run dev       # start the Vite development server
npm.cmd run build     # type-check and build the frontend
npm.cmd run tauri dev # start the desktop application
cargo check           # validate Rust code from tauri-app/src-tauri/
cargo test            # run Rust unit tests from tauri-app/src-tauri/
```

Use the project scripts rather than introducing parallel ad-hoc commands. PowerShell may require `npm.cmd` because the `npm.ps1` shim is restricted in this environment.

## Coding Style & Naming Conventions

Adopt the formatter and linter supplied by the selected toolchain, and run them before submitting changes. Use two or four spaces consistently (matching the language ecosystem), UTF-8 text, descriptive names, and small single-purpose modules. Use `PascalCase` for types/components, `camelCase` for variables and functions, and `kebab-case` for feature directories unless the framework requires another convention.

## Testing Guidelines

Add tests under `tests/` and mirror source structure where practical. Name tests after the behavior they verify (for example, `inspection-report.test.*`). Cover new behavior and regressions; add integration tests for boundaries such as persistence or external services. The eventual test command should be the same one used by CI.

## Commit & Pull Request Guidelines

No Git history is available yet, so existing commit conventions cannot be inferred. Use concise imperative subjects, optionally grouped by type (for example, `feat: add inspection report export` or `fix: validate empty asset code`). Pull requests should explain the change, include test results, link related issues, and attach screenshots or sample output for user-facing changes.

## Security & Configuration

Do not commit credentials, private keys, production data, or local environment files. Provide a documented `.env.example` for required settings, validate configuration at startup, and keep secrets in the deployment environment.
