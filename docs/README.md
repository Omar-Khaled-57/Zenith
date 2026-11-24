# Zenith documentation

Zenith is a small, transparent Tauri v2 desktop window that reports CPU
temperatures, CPU/RAM/disk usage, and the top processes. This index follows the
[hub documentation standard](../../docs/DOC-STANDARD.md).

## Start here

- [Product](./product.md) — what Zenith is for and what it shows today.
- [Architecture](./architecture.md) — how the React UI, Rust app, and sensor
  worker fit together.
- [Development](./development.md) — prerequisites, install, commands, and
  troubleshooting.

Configuration, tests, deployment, and release history each have their own page.
The pre-existing [visual identity guide](./visual-identity.md) is kept as the
design reference.

## Documentation index

| Page | Answers |
| --- | --- |
| [Product](./product.md) | Problem, audience, feature inventory, non-goals, known gaps. |
| [Architecture](./architecture.md) | Stack, entry points, process/thread model, event and sensor data flow. |
| [Development](./development.md) | Prerequisites, install, commands, repository map, conventions, troubleshooting. |
| [Configuration](./configuration.md) | Env vars, worker flags, config files, security notes. |
| [Testing](./testing.md) | Rust test locations, how to run them, coverage and gaps. |
| [Deployment](./deployment.md) | Build commands, artifacts, platforms, what is not configured. |
| [Changelog](./changelog.md) | Release-note pointers and release-history summary. |
| [Visual identity](./visual-identity.md) | Palette, surface treatment, and design source files. |
| [Hub documentation standard](../../docs/DOC-STANDARD.md) | The shared contract all hub projects follow. |
| [Hub design-system notes](../../docs/DESIGN-SYSTEM.md) | How project-specific visual docs are maintained. |

## Canonical source-of-truth files

| File | Authority for |
| --- | --- |
| [`package.json`](../package.json) | npm scripts, dependencies, product name, version. |
| [`src/main.tsx`](../src/main.tsx), [`src/App.tsx`](../src/App.tsx) | Frontend entry point and `system-metrics` event listener. |
| [`src/components/Dashboard.tsx`](../src/components/Dashboard.tsx) | Rendered features, status colors, thermal insights, source labels. |
| [`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json) | Window, CSP, build hooks, bundle targets. |
| [`src-tauri/tauri.linux.conf.json`](../src-tauri/tauri.linux.conf.json), [`src-tauri/tauri.windows.conf.json`](../src-tauri/tauri.windows.conf.json) | Bundled sensor-worker resource paths. |
| [`src-tauri/capabilities/default.json`](../src-tauri/capabilities/default.json) | Window and opener permissions. |
| [`src-tauri/Cargo.toml`](../src-tauri/Cargo.toml), [`src-tauri/worker/Cargo.toml`](../src-tauri/worker/Cargo.toml) | Rust crates and dependencies. |
| [`src-tauri/src/lib.rs`](../src-tauri/src/lib.rs), [`src-tauri/src/sensor.rs`](../src-tauri/src/sensor.rs), [`src-tauri/src/worker_supervisor.rs`](../src-tauri/src/worker_supervisor.rs) | App lifecycle, payload assembly, worker supervision. |
| [`src-tauri/worker/src/protocol.rs`](../src-tauri/worker/src/protocol.rs) | Versioned JSONL worker protocol. |
| [`CHANGELOG.md`](../CHANGELOG.md) | Author's development and release log. |
| [`public/icon.svg`](../public/icon.svg), [`src/App.css`](../src/App.css) | Product icon and glass surface styles. |

## Known gaps

- No `test` script is declared in `package.json`; automated checks are Rust
  tests, not npm tests. See [testing](./testing.md).
- No CI/CD or deployment configuration for Zenith itself was found. See
  [deployment](./deployment.md).
- No dashboard screenshot is committed; the visual identity page links the
  icon and source files instead.
