# Native mod loader: implementation roadmap

Each phase ends with working code + tests + a brief audit of changed engine lines. Do not skip phases by inventing unsupported Horizon APIs.

## Execution status — 2026-10-30

- Phase 0: local source audit and insertion ledger written in `HORIZON_INTEGRATION_MAP.md`.
- Phase 1: **gate CLOSED / PASS** against the mandatory Horizon regression/API criteria below. See `VALIDATION.md` for the evidence matrix. Full Minecraft Java parity is optional.
- Phase 2: **gate CLOSED / PASS** for the read-only Mods list/details and real static-item/creative slice. Embedded original resources feed the existing Horizon loaders; two independent API-only mods were rendered together. Real inventory/save/restart, one-to-two-mod ID migration, a pre-native vanilla world, resource reload and TCP mismatch refusal passed. See VALIDATION.md and PHASE2_SMOKE_TEST.md. Mutable native settings, dynamic models and creative pagination remain unsupported; no such capability is claimed by this gate.
- Phase 3: **gate CLOSED / PASS** for bounded gameplay snapshots/actions, blocks,
  shapeless recipes, versioned integer-field block entities and namespaced
  commands. Actual integrated/in-frame/LAN/dedicated scenarios and save/restart
  passed; see PHASE3_VALIDATION.md for results and inherited GPU-test failure.
- Phases 4–5: not started. No Create port work.

## Validation policy — decision of 2026-10-30

This project develops an independent Rust implementation of Minecraft. Absolute
equivalence to Minecraft Java is not a prerequisite for Native Mod Loader work.
Full vanilla differential parity, Java oracle suites and display-dependent vanilla
comparisons are **optional diagnostics**, not phase 1 blockers. Do not require or
install JDK 25 solely to close this gate. Keep upstream harnesses intact; report
their actual PASS/FAIL/NOT RUN results separately, without claiming full parity.

Mandatory: preserve existing Horizon behavior and tests, check native feature off
and on, retain compatibility with the existing WASM loader, validate lifecycle,
dependencies and errors, enforce API/engine isolation, and exercise registration
and mod interaction through the public API. A known Horizon regression is still a
blocker, even when found by an optional oracle. Phase-specific UI, persistence and
network checks remain required when those capabilities are introduced.

## Phase 0 — Audit, no engine edits

- Read upstream `horizon/AGENTS.md`, actual `Cargo.toml`, `docs/architecture.md`, `docs/mods.md`.
- Map startup, tick, creative inventory, menu/screen routing, items/blocks, serialization, network protocol, dedicated server, and WASM integration points.
- Create `HORIZON_INTEGRATION_MAP.md` with real paths and signatures, issues, possible hook points, and the smallest initial patch.
- Decide where the independent native API/runtime/pack crates live (prefer separate workspace or minimal explicit additions to Horizon's workspace), and how Cargo assembles a custom binary.

**Gate:** Architecture map reviewed; no speculative engine rewrites.

## Phase 1 — Compiled mods, metadata and lifecycle

- Validate and extend the standalone `native-mods/` MVP and its opt-in Horizon bridge. Implement minimal `horizon-native-api`: `ModInfo`, `ModId`, `Mod` or registration entry, `LoadContext` and scoped lifecycle hooks.
- Implement `horizon-native-runtime`: deterministic composition, dependency validation, errors, lifecycle subscriptions, and metadata query. Add mod pack manifest/generation later, not before a trivial static pack works.
- Implement single HelloWorld mod crate that depends only on the public API.
- Add smallest generic engine startup/shutdown bridge; opt in via feature/config. Preserve the existing Horizon no-mod path and current WASM support.

**Gate (mandatory):** native workspace checks/tests; Horizon regression tests with
`native-mods` off and on; combined `native-mods,mods` build and relevant WASM loader
tests; HelloWorld initialization/shutdown logs and metadata; deterministic public
registration/mod interaction; dependency/side/error/cleanup tests; dependency graph
review proving mods/API do not depend on engine internals. Feature-off behavior is
compared with existing Horizon, not with an absolute Minecraft Java specification.
No game assets are required for pure API/registry tests. Full vanilla parity and a
rendered game smoke test are supplementary for this startup-only slice; UI/world
validation belongs to the phases that add those capabilities.

## Phase 2 — Mod list, settings and creative tabs

- Build UI adapter that reads compiled `ModInfo` records and shows a Mods list + details, with the distinction from WASM clear.
- Implement namespaced item descriptors and tab descriptors through two-stage registration + freeze; adapt existing item/creative menu paths with generic data readers.
- Add a sample item in a mod-defined creative tab only after the engine's existing item ID/storage/render path is understood.

**Gate:** No per-mod match arms; two test mods both appear correctly; UI paths do not run on dedicated server.

**2026-10-30 acceptance: PASS.** Concrete composition is isolated in native-pack;
HelloWorld and opt-in Companion both render in the existing Mods and Creative UI.
Server-only runtimes produce no mod-list view and do not dispatch client_setup.
Registration/resource errors, stable save names, actual save/restart, old-world
compatibility, feature-off/on, WASM and matching/mismatching protocol identities
are validated in VALIDATION.md. Graphical checks used actual Vulkan offscreen
frames outside the test sandbox. Full dedicated-world, in-frame, visible-window
manual and Java differential runs are supplementary NOT RUN, not claimed passes.

The gate covers metadata/details and static content as specified above; it does
not invent native mutable settings or phase 3 gameplay capabilities. Phase 3 and
Create remain unstarted in this iteration.


## Phase 3 — Gameplay-level extension points

- Add opt-in hooks for player, world and block events via snapshots/action queues.
- Expand registries to cover blocks, recipes, block entities and commands as separate individually testable milestones.
- Validate save/reload, LAN/dedicated-server registry sync, mod dependency errors, and graceful mismatch handling.

**Gate:** Minimal example gameplay mod works end-to-end on both client and server.

**2026-10-30 acceptance: PASS.** Optional API-only Workshop works through the
public contracts in real offscreen clients and authoritative integrated, in-frame,
LAN and dedicated modes. Checked placement, actual crafting UI, player callbacks,
versioned data persistence, late join/live updates and mismatch refusal passed.
All registrations freeze before world entry. Existing Horizon off/on suites,
combined WASM checks and WASM runtime tests pass. Parallel renderer GPU tests
have the same SIGSEGV on the original phase 2 baseline; the serial renderer suite
passes. Exact evidence, commits and limitations: PHASE3_VALIDATION.md.

Initial contracts deliberately cover shapeless explicit-item recipes,
zero-argument player commands and single-state block entities with bounded signed
integer fields. Existing resource/Creative limits stay in place. No settings,
Creative expansion, dynamic models, automatic schema migration or Create is
required or claimed by this gate. Phase 4 is the next smallest roadmap stage.


## Phase 4 — Ecosystem & toolchain

- User-friendly `native-modpack.toml` and generator for `native-pack`; reproducible locked builds, dependency pinning, configuration/help, packaging without Mojang assets.
- Document API versioning and upgrade policy, testing templates, examples, performance budget and trusted-code warnings.
- Add CI for no-mod and modded variants; detect prohibited engine dependencies in mod crates.

## Phase 5 — Prepare for Create port (do NOT start Create yet)

- Audit required generics: block entities, fluid/energy capability model, kinetic networks, multiblock and moving contraptions, scripting recipes, animation, particles, render pipeline and server sync.
- Build one tiny independent example per generic API and prove nothing requires a mod-specific engine modification.

**Gate:** Portable APIs sufficient to implement a simple rotating machine mod without touching private engine logic.

## Change log template

For each PR/iteration record:

```text
Feature:
Public API delta:
Horizon file / exact hook:
Why this hook is genuinely necessary:
Alternative without engine edit:
No-mod and upstream compatibility:
Tests actually run:
Known limitations:
Next smallest task:
```
