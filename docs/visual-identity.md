# Zenith visual identity

Zenith is a compact hardware monitor with a dark, translucent desktop surface.
Cool blue marks telemetry and chart details; cyan communicates healthy thermal
state, while orange, magenta, and red indicate increasingly urgent conditions.

| Role | Color |
| --- | --- |
| Dashboard / glass surface | dark transparent surface with white low-opacity borders |
| Telemetry blue | ![#3b83f6](https://img.shields.io/badge/-%233b83f6?style=flat-square&color=3b83f6) `#3b83f6` |
| Healthy | ![#22d3ee](https://img.shields.io/badge/-%2322d3ee?style=flat-square&color=22d3ee) `#22d3ee` |
| Warning | ![#f97316](https://img.shields.io/badge/-%23f97316?style=flat-square&color=f97316) `#f97316` |
| Critical | ![#ef4444](https://img.shields.io/badge/-%23ef4444?style=flat-square&color=ef4444) `#ef4444` |

The scrollbar treatment is in [`src/App.css`](../src/App.css); thermal status
colors are semantic values in [`src/components/Dashboard.tsx`](../src/components/Dashboard.tsx).
Zenith does not currently have a dedicated screenshot in the repository.
