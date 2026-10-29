# First Codex task — copy into a new Codex conversation

Read the OUTER `AGENTS.md`, the local Codex skills and `docs/native-modding/{ARCHITECTURE,ROADMAP}.md` first. Then follow the upstream `horizon/AGENTS.md` and `CONTRIBUTING.md`.

Goal: Begin Phase 0 and, if audit confirms safe insertion points, implement only Phase 1's smallest working vertical slice: an opt-in native Rust mod loader whose mods are Cargo crates linked into a custom Horizon binary. Do NOT use WASM, source mixins or dynamic libraries for this path. Do NOT remove existing WASM mods.

Do not invent existing Horizon types/paths. Inspect the actual checkout, record exact callsites in `docs/native-modding/HORIZON_INTEGRATION_MAP.md`, and make a small plan before editing. Keep the native API free of engine-private dependencies. A sample `hello_mod` must register through the public API and show in a metadata list; generic bridge must dispatch a testable lifecycle event. Do not attempt item/creative registration until Phase 1 is verified; that is Phase 2.

Every engine change must be tiny, generic, and recorded in the change ledger with alternatives and tests. Never alter or commit `game-resources/` or Mojang data; run game tests with isolated temporary homes as required by upstream. Before exiting, report files edited, engine changes, tests actually run, and what remains for Phase 2. If Phase 1 cannot be integrated safely from the current checkout, complete Phase 0 and report the blocker instead of forcing a patch.
