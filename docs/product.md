# Zenith product

## The problem

Hardware telemetry is usually buried in a full settings or diagnostics app.
Zenith puts a small, always-glanceable set of readings in a compact transparent
desktop window and is explicit about where each temperature came from.

## Who it is for

The README describes a Windows 11 desktop monitor built for people who want
live CPU thermals and resource usage without a large dashboard. The release
notes add an audience of enthusiasts who care about per-core deltas, cooler
seating, and thermal paste condition.

## What it does today

Zenith is one window with no routes. Data arrives as a `system-metrics` Tauri
event and the UI renders the latest payload; there is no history view or
account state.

| Capability | Where it is implemented | What the code shows |
| --- | --- | --- |
| Temperature source transparency | [`src/components/Dashboard.tsx`](../src/components/Dashboard.tsx) (`SOURCE_LABELS`, `SourceBadge`) | Maps worker `hardware` to `REAL`, plus `ACPI`, `MOCK`, and `NONE` labels. |
| Dual-ring gauge | `Dashboard.tsx` (`GaugeSection`, `Ring`) | Package temperature on the outer ring, CPU usage on the inner ring, both on a 0–100 scale. |
| Per-core grid | `Dashboard.tsx` (`CoreGrid`, `tempColor`) | A four-column grid of core temperatures with status colors; hidden when no core data exists. |
| Thermal intelligence | `Dashboard.tsx` (`getInsight`) | Thresholds produce `THROTTLING RISK`, `REPASTE NOW`, `REPASTE SOON`, `UNEVEN MOUNT`, `ACPI FALLBACK`, `NO THERMAL DATA`, or `SYSTEM HEALTHY`. |
| Resource bars | `Dashboard.tsx` (`MetricBar`) | RAM and disk usage/total values in GB supplied by the Rust backend. |
| Top processes | `Dashboard.tsx` (`ProcessList`), [`src-tauri/src/sensor.rs`](../src-tauri/src/sensor.rs) | Backend sorts by CPU share and truncates to five; UI displays up to five rows with CPU and memory. |
| Elevation prompt | `Dashboard.tsx` (`ElevationBanner`), [`src-tauri/src/lib.rs`](../src-tauri/src/lib.rs) | When the source is `none`, a banner invokes `restart_elevated`. |
| Window controls | [`src/App.tsx`](../src/App.tsx) | Minimize and close buttons call Tauri window APIs; the title bar is a drag region. |
| Render safety | `Dashboard.tsx` (`ErrorBoundary`) | A render failure shows `DASHBOARD ERROR` with a `RETRY` button. |

Temperature behavior is defined in [`src-tauri/src/sensor.rs`](../src-tauri/src/sensor.rs):
a healthy worker sample wins; otherwise a labeled ACPI thermal-zone reading is
used; otherwise the payload reports `none`/`unavailable` with a worker error.
The mock backend in
[`src-tauri/worker/src/mock.rs`](../src-tauri/worker/src/mock.rs) is labeled
`MOCK` in the UI and is described as development-only.

## Non-goals and known gaps

- Not a benchmarking, logging, or alerting service: no database, no history,
  no notifications, and no persisted settings were found in the source.
- The README and [1.1.0 release notes](../dev/release/1.1.0/RELEASE_NOTES.md)
  record Intel-only hardware decode; AMD decoders, hybrid Intel CPUs, and
  >64-thread processor groups are listed as open limitations. The
  [macOS backend](../src-tauri/worker/src/macos_stub.rs) is a placeholder that
  reports the driver as unavailable; the Linux backend reads hwmon sensors.
- No npm test script exists; see [testing](./testing.md).
- No dashboard screenshot is committed; see
  [visual identity](./visual-identity.md).

## Sources

[`README.md`](../README.md), [`src/App.tsx`](../src/App.tsx),
[`src/components/Dashboard.tsx`](../src/components/Dashboard.tsx),
[`src-tauri/src/sensor.rs`](../src-tauri/src/sensor.rs),
[`src-tauri/src/lib.rs`](../src-tauri/src/lib.rs),
[`src-tauri/worker/src/mock.rs`](../src-tauri/worker/src/mock.rs),
[`CHANGELOG.md`](../CHANGELOG.md),
[`dev/release/1.1.0/RELEASE_NOTES.md`](../dev/release/1.1.0/RELEASE_NOTES.md).
