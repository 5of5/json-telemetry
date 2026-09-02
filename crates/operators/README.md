# Operator binaries

535 distinct crates. Each crate is one Binary Repository v1 operator. AriA (`telemetry::transform`) is linked into every binary via `aria-operator`. The closed operator JSON is unique; the nested `telemetry` object is the shared `aria-telemetry-query-v1` spine.

Regenerate: `python3 crates/aria-operator/generate.py`
