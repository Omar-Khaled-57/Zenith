# Zenith changelog

The authoritative running log is [`CHANGELOG.md`](../CHANGELOG.md) at the
project root. Per-release notes are kept in the ignored `dev/release/` folder:

| Version | Release notes |
| --- | --- |
| 1.0.0 | [`dev/release/1.0.0/RELEASE_NOTES.md`](../dev/release/1.0.0/RELEASE_NOTES.md) (also installer files for MSI and NSIS) |
| 1.1.0 | [`dev/release/1.1.0/RELEASE_NOTES.md`](../dev/release/1.1.0/RELEASE_NOTES.md) (notes only; no 1.1.0 installer files in the tree) |

## Release-history summary

- **0.1.1 — Code quality and stability:** TypeScript cleanup in `App.tsx`,
  dashboard component extraction, Tailwind v4 CSS-first configuration replacing
  the old config files, and more realistic simulated readings at the time.
- **1.0.0 — Production release:** real ACPI package temperature on Windows,
  per-core data shown only when available, first-payload accuracy fixes,
  animated gauges, a restrictive CSP, graceful sensor-thread shutdown, and
  installer artifacts.
- **1.1.0 — Real CPU temperatures:** production simulation removed, package and
  per-core readings decoded from Intel MSRs by `zenith-sensor-worker` through
  PawnIO, `REAL` / `ACPI` / `MOCK` / `NONE` source labels, an elevation prompt,
  the versioned JSONL worker protocol, and debug-only worker path/mock
  overrides.

[`package.json`](../package.json) and [`src-tauri/Cargo.toml`](../src-tauri/Cargo.toml)
both declare version `1.1.0`. The changelog has no separate "Unreleased"
section. Numbers and validation results quoted in the notes (for example the
steady-state RSS figure, live temperature range, and test count) are the
author's recorded observations, not measurements produced by this
documentation pass.
