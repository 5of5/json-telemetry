# work — the worker gateway

PCVC keeps a folder per feed. This repo keeps a **crate per binary**.
Repeats are fine. Exact compilation beats a jammed mega-target.

Workers do not import 535 packages. They pass work here:

```bash
work --binary BIN.PEOPLE --in payload.json
work --commands                         # hosted command list (JSON API)
echo '{"work":"BIN.PEOPLE","in":{"nodes":[{"id":1,"type":"Person","label":"Ada"}]}}' | work --json
echo '{"ops":["BIN.PEOPLE","BIN.COMPANY"],"in":{"nodes":[...]}}' | work --json
```

One JSON telemetry base (`aria-operator`). One gateway (`work`). 535
separate `src` programs under `crates/operators/`. The gateway can grow
(transports, plan_hash, Observe-first) without merging those crates.

`--telemetry` embeds the Aria spine. Off by default (sheet 09 optional).
