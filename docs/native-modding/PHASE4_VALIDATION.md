# Phase 4 — ecosystem/toolchain acceptance (2026-10-30)

Gate: **CLOSED / PASS** for all three phase 4 ROADMAP bullets. Generated source
recipes, locked builds, portable packaging, author/version/performance guidance
and CI/boundary definitions are implemented and locally validated. Hosted Actions
execution remains an explicit external follow-up, not a claimed green run.
No Create work or new gameplay API is included.

## Delivered contracts and engine audit

- Strict schema-1 native-modpack.toml generates the existing native-pack crate.
  Exact local package versions, reviewed workspace paths, explicit Cargo features,
  deterministic composition and unchanged public pack entrypoints. Empty recipe,
  default Hello, independent Companion and Workshop compositions are supported.
- native-modpack.lock pins API epoch, Rust/Cargo identity, host triple, Horizon
  f84396c94, both Cargo.lock files, tracked native sources and build/generator/graph
  tools. Explicit lock updates are distinct from read-only verify; builds use
  locked/offline Cargo and produce binary SHA-256 receipts.
- Source recipe ZIP exports an explicit Git-tracked allowlist; fixed member order,
  timestamps and modes, BUILD.json and SHA256SUMS. No engine checkout, binaries,
  player data, caches or Mojang resources. Real roundtrip checks the lock and
  builds the exported API/runtime/examples in a temporary clean checkout.
- API_POLICY.md documents experimental source epoch/upgrade, lifecycle, stable
  save identities and multiplayer compatibility. TOOLCHAIN.md covers all commands,
  configuration/help, intentional upgrades, packaging and trusted-code warnings.
  PERFORMANCE.md defines targets and the actual runtime-only diagnostic.
- API-only author template compiles/tests independently. Existing independent
  Hello/Companion/Workshop examples validate actual generated compositions and
  authoritative client/server behavior. No mod depends on engine/runtime/UI.
- Resolved native Cargo graph guard walks all dependency kinds/platforms, including
  renamed/transitive edges; prohibits engine/private native layers, escaping local
  paths and floating registry/git declarations. API remains std-only. Feature-off
  production graph is checked separately via targeted cargo tree.
- CI has tooling/API checks and vanilla/native/combined-WASM Horizon jobs; renderer
  tests use one thread. It checks out the exact engine revision with no fallback.

Public Mod API delta: **none**. API_VERSION remains 1, crate source version 0.1.0.
Runtime/adapter/game/save/network implementations: **unchanged**. Only pack
composition source/Cargo declarations are generated; the feature names and
constructor behavior remain compatible. Engine edits: **zero**, so no engine
commit. HORIZON_INTEGRATION_MAP.md records why existing boundaries suffice.
Existing vanilla content, WASM runtime and parity harnesses are unchanged.

## Checks actually run

Selected toolchain: rustc 1.99.1 / Cargo 1.99.1, x86_64-unknown-linux-gnu,
Python 3.14.8. Cargo checks/tests use --locked --offline. Test/game environments
use scratch game/launcher/XDG homes, launcher scratch trash/clipboard/desktop,
no audio/self-update/Discord/telemetry. Player-prepared resources were read-only
for actual rendered scenarios; never used for unit fixtures or packaged.

| Check | Actual result |
|---|---|
| Python generator/release/graph tests | PASS: 19; malformed configuration/locks, version/API mismatch, duplicate/normalized aliases, symlinks/traversal, stale generation/hash drift, prohibited dependency paths, deterministic packaging/checksums/no overwrite/dirty refusal. |
| Generated zero/one/two/Hello+Workshop packs | PASS: four actual scratch compilations and host integration tests; each checks Client/Server/Combined, freeze/setup, stable content and repeated shutdown. |
| Independent author template | PASS: 3 tests (template + API), all-target Clippy -D warnings. |
| Native workspace all-feature tests | PASS: 29, including dependency/side/lifecycle/error/registration and gameplay examples. |
| Native workspace fmt, all-target/all-feature Clippy -D warnings; template rustfmt | PASS. |
| Native graph + feature-off host production graph | PASS: three API-only mod crates, std-only API; no native crates on feature-off path. Synthetic renamed/transitive/build/dev/Windows-target prohibited edges fail. |
| Verified wrapper vanilla build (client + ctl) | PASS; warm incremental engine compilation 27.76 s. |
| Verified wrapper native + gameplay-example build | PASS; 20.32 s after pack regeneration, later warm build 0.54 s. Receipts record selected variant/features and binary hashes; these are not cold build benchmarks. |
| Horizon feature-off lib/bin/ctl tests | PASS: 669 lib, 153 game bin, 18 ctl; 21 lib + 3 game bin ignored. |
| Horizon native + gameplay-example lib/bin/ctl | PASS: 677 lib, 160 game bin, 18 ctl; same ignores. Includes 8 native adapter regressions and actual save/world identity tests. |
| Combined native-mods,mods,gameplay-example binary check | PASS. |
| Combined native/WASM lib/bin/ctl | PASS: 677 lib, 163 game bin, 18 ctl; 21 lib + 3 game bin ignored. |
| Original WASM runtime,pack tests | PASS: 22 unit + 4 API reference + 32 real sandbox; 1 performance ignored. |
| Sequential renderer suite | PASS: 526; 15 ignored; 35.97 s, --test-threads=1. |
| Horizon native lib/game-bin Clippy -D warnings | PASS. |
| Source ZIP deterministic bytes + checksum coverage | PASS: two actual exports compared equal; each member checksum verified. |
| Actual package -> new Git checkout -> exact local engine checkout -> verify/graph/native tests | PASS: roundtrip after correcting the missing graph-checker export; 29 tests in exported workspace. |
| CI YAML parse | PASS locally. Hosted Actions/actionlint NOT RUN (see limits). |
| Diff whitespace / public tree guard | PASS; Horizon reports Public tree check passed. |
| Full Java differential/asset oracle parity | NOT RUN; supplementary under ROADMAP policy, no engine gameplay changes. |

## Real generated-pack integration

Actual NVIDIA Vulkan offscreen frames, generated Hello+Workshop pack, temporary
homes. The same native executable was built with the verified wrapper. The reused
phase 3 harness retains its phase3 scratch-directory names; these runs happened
again during phase 4.

| Scenario | Actual result |
|---|---|
| Integrated world + actual crafting + player break/place callbacks | PASS; ordinary UI crafted workshop:counter and the public callbacks reacted to successful block operations. |
| Same-home integrated save/restart | PASS; counter at [2,80,0] restored with stable workshop:counter_data version 1 and visits >= pre-shutdown value. |
| F3+T + Mods UI | PASS; retained entity payload, no repeated Hello initialization; both native rows remained visible. |
| TCP absent/different static identity | PASS; both local protocol probes explicitly refused before play. |
| LAN matching guest late join + live data/commands | PASS; guest received existing/increasing counter data and its new placement appeared on host. |
| In-frame crafting/block events/save/restart | PASS; same counter position/type/version/fields restored after normal shutdown. |

Repeat commands:

```sh
python3 tools/native_pack_release.py build --variant native --features gameplay-example
python3 tools/native_phase3_smoke.py --binary "$CLIENT" --ctl "$CTL" --resources "$READ_ONLY_RESOURCES" --mode integrated --reload
python3 tools/native_phase3_smoke.py --binary "$CLIENT" --ctl "$CTL" --resources "$READ_ONLY_RESOURCES" --mode in-frame
python3 tools/native_package_smoke.py
```

CLIENT/CTL are absolute paths to the wrapper's built binaries. READ_ONLY_RESOURCES
is supplied privately; no resource path/bytes or saves are distributed. Smoke
helpers create the isolated homes themselves. Integrated now invokes the existing
bounded loopback mismatch probe before the matching guest joins.

## Failures investigated and resolved

- First feature-off regression run inside the restricted sandbox: 663 passed,
  6 failed with Operation not permitted creating loopback/Unix sockets. Repeated
  unchanged suites outside those restrictions with scratch homes: all pass.
- Full-workspace Horizon cargo metadata offline attempted uncached Android and
  unrelated workspace packages. The feature-off check now uses targeted
  horizon-client production cargo tree. Native graph audit retains full unfiltered
  metadata so inactive-platform dependencies cannot evade the mod boundary.
- Initial generated Rust indentation failed fmt; corrected before generator
  implementation commit. Final generated files pass both check and rustfmt.
- Real source ZIP roundtrip found check_native_boundaries.py omitted by the
  native*.py allowlist. Fixed in its own commit; added permanent actual package
  roundtrip harness and a member regression assertion. The corrected roundtrip
  passes without weakening the lock or relocking extracted sources.
- Review added fail-fast reserved/native layer names, normalized Rust crate alias
  collisions, linked Cargo.toml rejection and malformed lock diagnostics, each
  with regression coverage and atomic commits.
- Parallel GPU SIGSEGV remains the inherited diagnosis from PHASE3_VALIDATION.md:
  default-parallel current and phase-2 fd7316fd1 renderer suites both reproduced
  it there. Phase 4 did **not** attempt a fix or rerun the known parallel failure;
  its actual sequential renderer result is the passing regression evidence.

## Limits and remaining external checks

- Hosted CI has been added and its local-equivalent checks run; no remote Actions
  run is claimed. Set NATIVE_HORIZON_REPOSITORY to a repository exposing the pinned
  integration commit before running hosted Horizon jobs. Local engine integration
  commits are not published by this task; no push was authorized/performed.
  actionlint is unavailable locally; YAML parsing is not full Actions validation.
- Reproducibility pins source/dependency selection and toolchain on the recorded
  host, and exact source ZIP bytes. Cross-host/path/linker/environment bit-identical
  executables are not claimed. Rust build scripts/configuration remain trusted.
- Recipe supports reviewed local mod sources, not automatic arbitrary remote mod
  discovery/install or binary plugins. Third-party Rust dependencies require exact
  pins/lock checksums and a prefetched cache for offline builds. The recipe ZIP
  requires the exact engine checkout and separately prepared licensed resources.
- Native code is unrestricted trusted process code, not a sandbox. Hashes and the
  graph guard do not prove safety, source rights, determinism or save compatibility.
- No new dedicated gameplay implementation was added; dedicated save/restart and
  side separation remain established by phase 3 and three-side generated runtime
  checks. A fresh dedicated/EULA scenario is NOT rerun in phase 4. Native-disabled
  old-world/native-world refusal remains covered by unchanged unit regressions and
  the actual phase 3 acceptance, not claimed as a new phase 4 game run.
- Runtime mutable settings, expanded Creative capacity, schema migrations,
  dynamic models, machine capabilities and Create remain unsupported/out of scope.

## Atomic commits

Outer repository starts at fd061d4 (parent d3e885b). Implementation commits:
cf50d35 generator/recipe; b36cac4 lock/build/package; d70e807 boundaries/CI;
c574fa9 policy/template/performance; 64c0fa7 alias/manifest validation;
ad481f8 portable guard + roundtrip; da7783f WASM/packaging CI + mismatch smoke;
ed0a019 malformed-lock diagnostics. Final acceptance documentation is separate.
The final response reports the inclusive range through that final commit.
Horizon remains at f84396c94, with zero phase 4 commits. Pre-existing untracked
user files in both repositories are excluded. No push. Next smallest external
step: make the pinned integration revision available to configured hosted CI and
observe its run; no Create or phase 5 implementation starts automatically.
