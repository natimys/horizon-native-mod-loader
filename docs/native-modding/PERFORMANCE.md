# Native author performance budget

These are engineering targets, not enforced fuel/memory limits. Trusted native
code can exceed any budget, block or abort. Profile on the target host with a
matching pack, save and player count; synthetic runtime dispatch is not FPS or
whole-world performance evidence.

- No subscription means no callback and no per-frame idle polling. Subscribe only
  to the events needed. Client hosts execute no authoritative gameplay callbacks.
- Aim for <=100 microseconds per ordinary callback and <=1 ms aggregate native
  work per authoritative 50 ms tick on the measured host. At 1000 ticking entities,
  a 100 us per-entity callback is already too expensive: reduce frequency/work.
  These are review targets, not universal pass/fail CI timings.
- Keep register/freeze/setup work outside world ticks. Initial target <=10 ms per
  tiny example's runtime-only startup, excluding resource decode/GPU/engine startup.
- Existing public hard bounds remain 256 actions/callback, 4096 bytes/message,
  <=64 integer fields/entity; host pending queues <=4096. Reaching a bound is a
  correctness/error-path concern; it is not permission to fill it every tick.
- Do not perform file/network/process/blocking work inside callbacks, repeatedly
  clone all snapshots, force-load chunks or resubmit unchanged state. The API
  offers no custom background engine handles or global pointers.

Repeat the diagnostic on the selected toolchain/host:

```sh
cargo run --manifest-path native-mods/Cargo.toml -p horizon-native-pack --example budget --release --locked --offline
cargo run --manifest-path native-mods/Cargo.toml -p horizon-native-pack --example budget --release --features gameplay-example --locked --offline
```

It performs 100,000 WorldTick dispatches over an owned empty-player snapshot,
using black_box, and measures runtime initialize (registration/freeze/setup).
Construction/metadata resolution, host snapshots/actions, resource loading,
entity ticks and renderer costs are excluded. Empty and client cases deliberately
call dispatch to expose its lower-bound cost; the real host skips unsubscribed
work. No timing assertions enter CI.

2026-10-30 local x86_64-unknown-linux-gnu, rustc/Cargo 1.99.1, optimized:
Hello-only empty/client/server dispatch 13 ns, startup 2/14/6 us.
Hello+Workshop empty/client/server dispatch 15/15/45 ns, startup 7/30/26 us.
These single-run diagnostics confirm the tiny examples are below the stated
runtime-only targets, not a stable benchmark or an engine-wide speed claim.
Warm Horizon build/check durations and exact regression commands are recorded
separately in PHASE4_VALIDATION.md. No baseline GPU fix or renderer optimization
is part of phase 4.
