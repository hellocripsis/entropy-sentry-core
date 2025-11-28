# entropy-krypton-core

`entropy-krypton-core` is a small Rust library that takes simple telemetry metrics, turns them into signals, and runs them through a Sentry engine that decides whether to keep, throttle, or kill jobs.

It’s designed as a lightweight example of how to turn noisy metrics into clear decisions in a backend/system context.

---

## Features

- Compute basic metrics from samples (`EntropyMetrics`)
- Turn metrics into normalized signals (`SentrySignals`)
- Configurable policy engine (`SentryEngine`) with:
  - `Keep`
  - `Throttle`
  - `Kill`
- Behavior-based tests for low / medium / high stress scenarios
- Demo binary that prints decisions for a series of synthetic jobs

---

## Getting started

### Run tests

```bash
cargo test
