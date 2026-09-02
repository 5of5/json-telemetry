# work — the worker gateway

PCVC keeps a folder per feed. This repo keeps a **crate per binary**.
Repeats are fine. Exact compilation beats a jammed mega-target.

Workers do not import 535 packages. They pass work here:

```bash
work --binary BIN.PEOPLE --in payload.json
work --operator PEOPLE --in payload.json
work --list
```

One JSON telemetry base (`aria-operator`). One gateway (`work`). 535
separate `src` programs under `crates/operators/`. The gateway can grow
(transports, plan_hash, Observe-first) without merging those crates.

`--telemetry` embeds the Aria spine. Off by default (sheet 09 optional).
