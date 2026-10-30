# Phase 4 — toolchain plan (2026-10-30)

Observed: phase 3 is accepted in PHASE3_VALIDATION.md. The engine consumes only
`horizon-native-pack::{create_runtime,create_for_side}` at the optional relative
path `native-mods/pack`; its current features select independent API-only examples.
API epoch is `API_VERSION = 1`; Cargo source version is 0.1.0. Native workspace is
std-only. Both native and Horizon lockfiles are tracked. No phase 4 gate is stated
separately in ROADMAP; acceptance therefore requires all three phase 4 bullets.

1. Add a strict versioned `native-modpack.toml` and deterministic developer-side
   generator, retaining the existing pack entrypoints/features. Pin local crate
   versions and reviewed source hashes, Rust/Cargo versions, both Cargo.lock files
   and Horizon revision in an explicit lock receipt. Generation/checks must not
   execute mod constructors or modify engine source. Test malformed configuration,
   zero/one/two/gameplay compositions and generated runtime lifecycle/side behavior.
2. Add a resolved Cargo graph boundary check and CI for feature-off/on, combined
   WASM, independent examples and sequential renderer regression. A transitive,
   renamed or target/build-only engine dependency must fail. No new game APIs.
3. Add deterministic source recipe packaging from an explicit tracked-file list,
   with source/hash verification and no resources, saves, build output or secrets.
   Document exact upgrades, author tests, build configuration, performance budgets,
   trusted code and platform/rebuild/multiplayer limitations.
4. Run actual local tooling/API/regression checks and relevant generated-pack
   integration scenarios; document PASS/FAIL/NOT RUN and close only when all phase
   4 requirements have evidence. Preserve the parallel GPU baseline diagnosis;
   use the sequential renderer suite. No Create work.

## Candidate engine insertions / ledger

None. The existing composition path and public pack functions are sufficient.
Alternative of adding engine configuration or concrete-mod branches is unnecessary.
Horizon source changes require a separate concrete need and a ledger entry first.

## Commit units

Audit/plan; manifest/generation with tests; lock/build/package tooling with tests;
graph guard/CI with tests; author/version/performance documentation and templates;
final validation and acceptance. Independently completed corrections get their own
commits. Existing untracked user files are excluded. No push.
