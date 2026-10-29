# Latest phase 3 evidence

Phase 3 gate is **CLOSED / PASS** for its bounded gameplay API. See
[PHASE3_VALIDATION.md](PHASE3_VALIDATION.md) for current contracts, exact engine
changes, real integrated/in-frame/LAN/dedicated tests, save/restart and mismatch
checks, atomic commits and the independently reproduced baseline GPU-test failure.
The phase 2 evidence below is historical and remains valid for that stage.

# Latest phase 2 real-item validation — 2026-10-30

This section supersedes earlier "content not integrated" and compositor-blocked
notes below. Earlier entries remain historical evidence, not current status.

**Phase 2 gate: CLOSED / PASS** for the read-only Mods list/details and static
item/creative content defined by ROADMAP.md. Two real independent API-only mod
crates were rendered in one client; ID, resource, inventory, persistence and
mismatch behavior were exercised. Phase 3 and Create remain unstarted. Mutable
native settings are not implemented or claimed by this gate.

## Mandatory gate and implementation evidence

| Criterion | Actual evidence / result |
|---|---|
| Independent mod -> public API -> real item | PASS: HelloWorld registers hello_mod:token, tab, original texture/model and EN/RU files through LoadContext only. Default pack is one mod; optional test-pack adds Companion with its own original resources. Both mod crates depend only on native-api. |
| Reuse existing systems | PASS: native_content translates to Content::read + mod_content::resource_root; ordinary Items, model/PNG, language, Creative, commands, Builtin protocol tables and item NBT remain authoritative. No new live item registry/renderer/save codec. |
| Two independent mods appear correctly | PASS: actual Vulkan offscreen client rendered both Mods rows and both Creative tabs/icons. Hello Token and Companion Marker were taken through Creative into separate hotbar slots; control inventory reports both namespaced IDs and count 1. Existing vanilla tabs/icons remained visible. |
| Server/client separation | PASS: Client/Server/Combined lifecycle tests, server-only UI snapshot is None, fixed execution-role tests, two-mod runtime tests on all hosts; headless builds. No native client_setup/mod-list renderer is dispatched on a server host. Common resource/tab validation is data preparation, not mod UI rendering. |
| Stable IDs, registration/composition order | PASS: two API-only producers through Runtime::freeze and existing host loaders, reversed constructor order, separate cache roots -> same item protocol mapping/content identity. Saves store strings. Real migration from one to two compiled mods shifted Hello Token protocol ID 1658 -> 1659; saved hello_mod:token and count 1 survived. |
| Item save/reload | PASS: a real Creative-taken item was saved on the integrated server, process exited normally, single-mod native binary reopened the same scratch world and restored the same ID/count/slot. Gzipped temporary players/data NBT contains hello_mod:token. No save codec changes. |
| Existing vanilla/content compatibility | PASS: old vanilla inventory roundtrip and a pre-existing content namespace/item keep names, IDs and translations in adapter tests; base resource files/marker remain unchanged. Full feature-off suite and real feature-off game startup passed. Real VanillaBefore world was created/saved by the feature-off binary, then reopened by the default native binary: minecraft:stick count 3 remained in slot 0; integrated join had zero decode errors. |
| Invalid registrations/resources | PASS: runtime rejects duplicate item/tab/resource IDs, foreign/reserved namespaces, traversal, undeclared dependencies and side-specific persistent content before publication/setup. Host rejects missing templates, model/texture/translation, foreign-owner translation, item/namespace collision; creative rejects duplicate/special/overflow categories atomically. |
| Protocol mismatch | PASS: memory configuration tests cover matching/different/absent/skipped confirmation. Real integrated join succeeds. Real TCP LAN login/configuration probes with no identity and wrong identity both receive explicit static-content Disconnect before play. No numeric remapping or downloads are offered. |
| Resource reload/localization | PASS: rendered EN and RU titles/icons, ordinary F3+T reload, native tab/icon/item retained afterwards. Exactly two HelloWorld initialize messages (client + integrated server), with none replayed by reload. |
| Existing WASM loader | PASS: combined native-mods,mods checks including two-mod pack; horizon-mods runtime/pack tests: 22 unit + 4 API reference + 32 real sandbox tests. 1 performance test ignored. No WASM production source modified. |
| No per-mod engine branches | PASS: concrete HelloWorld/Companion composition stays in native-pack. Engine adapter reads generic frozen descriptors. Save/item/renderer implementation modules and source-mixin hooks unchanged; binary hookability regression test passes. |

## Commands actually run

Commands from the outer root use --manifest-path; Horizon-only commands can also
run in horizon/. Cargo tests and all launched games used isolated home/XDG/launcher
paths, no self-update/audio/Discord/telemetry, and no personal data. The launcher
scratch trash/clipboard/desktop paths were set. XDG_RUNTIME_DIR was preserved.

| Command/check | Result |
|---|---|
| Native workspace cargo test --workspace --offline | PASS: 21 tests, plus doc-test targets |
| Native workspace cargo test --workspace --all-features --offline | PASS: 22 tests, including optional two-mod composition on all host sides |
| Native cargo check --workspace --all-targets --all-features --offline | PASS |
| Native cargo clippy --workspace --all-targets --all-features --offline -- -D warnings; cargo fmt --all --check | PASS |
| Horizon cargo test -p horizon-client --lib --bin horizon-client --no-default-features --offline | PASS outside sandbox: 666 lib + 153 binary; 21 lib + 3 binary ignored |
| Horizon cargo test -p horizon-client --lib --no-default-features --features native-mods --offline | PASS outside sandbox: 670 lib, 21 ignored (before final additional legacy-content test); final affected adapter tests rerun below |
| Horizon native feature binary tests | PASS: 159, 3 ignored |
| Final native_content::tests:: library filter | PASS: 5 tests, with real API-only registration, reversed composition, NBT/loaders, resources/errors, static digest and prior content compatibility |
| Two-mod native binary tests, filter native_ | PASS: 9 including bridge, catalog and source-mixin hookability |
| Horizon cargo check --bin horizon-client --no-default-features --features native-mods,mods --offline | PASS; repeated with horizon-native-pack/test-pack and --locked |
| Horizon cargo build --bin horizon-client --bin horizon-ctl --no-default-features --features native-mods --locked --offline | PASS; default one-mod binary restored after acceptance pack test |
| Same build with horizon-native-pack/test-pack | PASS; actual two-mod rendering and migration smoke |
| Horizon feature-off --bin horizon-client build --locked --offline | PASS; actual feature-off game/world creation |
| Horizon cargo clippy --lib --bin horizon-client --no-default-features --features native-mods --offline -- -D warnings | PASS |
| Final Horizon cargo clippy --lib --tests --no-default-features --features native-mods --offline -- -D warnings | PASS after fixing one test-only cloned-ref assertion |
| cargo test -p horizon-mods --features runtime,pack --locked --offline | PASS: 22 + 4 + 32; 1 ignored |
| Feature-off normal Cargo dependency graph | PASS: no horizon-native-* crates, including Companion |
| Cargo metadata boundary review | PASS: API has no dependencies; HelloWorld/Companion only native-api; runtime only native-api; pack owns composition; ui only API/runtime |
| Targeted rustfmt --check on changed engine Rust files; git diff --check in both repositories; tools/check_public_tree.py | PASS |

Initial full native library run inside the sandbox: 664 passed, 6 failed, 21
ignored. All six failures were existing local TCP/Unix-socket tests (control,
Discord fake socket, resource-pack download test HTTP servers). The unchanged
suite passed 670/0/21 outside the sandbox. These initial failures are not hidden
or counted as passes. A test-only Clippy warning was separately corrected and
committed. The original public descriptor-extraction signature was restored as
a compatibility wrapper; resource-aware extraction is additive. Native all-feature
tests (22) and Clippy passed again after that adjustment. The initial GUI probe also timed out its control request before GPU
initialization finished; the supervisor was corrected to wait for readiness and
subsequent runs passed. No engine change was made to bypass display access.

## Wayland and real-client results

The socket exists at the session's XDG_RUNTIME_DIR. In sandbox, connect returns
EPERM; outside sandbox, the same connection succeeds. Isolated clients rendered
real 640x480 Vulkan frames on the session GPU with --offscreen. Visual inspection
confirmed original orange H and blue C icons, separate tabs, English/Russian
labels, two Mods rows and read-only details. This replaces the earlier unresolved
"Could not find wayland compositor" status; the failure was environmental.

All saves, options, caches, screenshots and LAN listeners belonged to fresh /tmp
scratch games. Prepared resources were read-only inputs. Screenshots/derived game
caches are not committed or uploaded. Clients and temporary LAN listeners were
closed normally after validation. No personal saves/settings were accessed.

Observed debug-build resource startup (not a controlled benchmark): feature off
17.1 s / 1658 items; single mod 39.8 s / 1659 items; two mods 31.9 s / 1660 items.
First rendered frame setup after resources: 7.307 / 7.837 / 7.097 s respectively.
Build observations: feature-off 52.03 s and two-mod 51.01 s for the recorded
incremental feature rebuilds; cache/compiler states differ. These numbers do not
establish a regression budget. Native overlays use the existing table pipeline
and have material startup/cache I/O cost; no native per-frame polling or tick
callback was added. No native gameplay ticks are dispatched yet.

Supplemental NOT RUN: ordinary visible-window interaction, in-frame world smoke,
full dedicated-world smoke, full Java differential parity, and ignored upstream
full-resource-root/performance tests. Headless builds/role tests and real default
server-thread client-world/persistence/TCP checks did run. Mutable native settings,
dynamic/special models, creative pagination and arbitrary registry sync are not
implemented. These are not claimed as working.

Manual reproduction: PHASE2_SMOKE_TEST.md. Next smallest development stage: a
separate scoped proposal for the first phase 3 snapshot/action hook (or native
configuration contract); no phase 3 code or Create port was started here.

Commit ranges for implementation: outer repository `845c45a`–`fafd387` inclusive;
Horizon repository `30cc874de`–`fd7316fd1` inclusive. The following acceptance-docs
commit records the final gate/manual instructions. No push was performed. Existing
untracked user files in both repositories are excluded from all stage commits.

---

# Native loader validation — 2026-10-30

## Latest increment — existing Mods screen integration and creative insertion

Horizon commits (inclusive): `1f92f3e8b`–`859e73057` (four commits).
`e79d9f260` records only the explicitly authorized pre-existing phase 1 engine
baseline. The three new engineering steps are read-only Options list/details
(`1f92f3e8b`), runtime snapshot projection (`74cfb34af`), and generic creative
category insertion/search (`859e73057`). The outer integration plan is `820dca4`;
this report accompanies a separate documentation commit. Nothing was pushed.

Public API delta: no change to the mod-facing native-api/runtime/ui crates.
Host presentation adds `CatalogMod`, `ModList::catalog`, `Page::ModDetails`;
`Creative::append_tabs(Vec<Tab>)` is a generic host insertion boundary.
Production metadata comes only from `NativeRuntime::catalog()` -> public
`ModListView`, never a parallel manifest or another pack construction. The
projection includes ID/name/version, authors/license/description, API/side,
exact dependencies and optional absence, host selection and runtime-wide state.
Font wrapping uses Horizon's current Font. Details have no settings/toggle
callbacks; existing WASM/mixin installed indices and toggle persistence stay
separate. The engine ledger lists every insertion and its alternative.

| Check actually run | Result |
|---|---|
| `cargo test --manifest-path native-mods/Cargo.toml --workspace --offline` | PASS: 18 tests, plus doc-test targets |
| Native workspace `cargo fmt --all --check` | PASS |
| `cargo test -p horizon-client-render --no-default-features --offline --lib options_ui::tests::` | PASS: 32; 1 ignored (prepared language data) |
| Same render target, `mod_list::tests::` | PASS: 4 |
| Client binary tests, `--no-default-features --features native-mods --offline` | PASS: 157; 3 upstream ignored |
| Client binary tests, `--no-default-features --offline` | PASS: 153; 3 upstream ignored |
| Native catalog projection test after adding long localized text and empty snapshots | PASS: 1; same runtime state and repeated snapshot, two public API mods on Client/Combined/Server |
| Native-off `cargo tree -e normal --prefix none` | PASS: no horizon-native-* dependencies |
| Client binary `cargo check --no-default-features --features native-mods,mods --offline` | PASS: both loader features enabled |
| `cargo test -p horizon-mods --offline --lib` | PASS: 18 packaging/manifest/content tests |
| `cargo test -p horizon-mods --features runtime,pack --offline` | PASS: 22 unit + 4 API reference + 32 sandbox tests; 1 sandbox ignored |
| Client `--lib` tests, `--no-default-features --offline creative::tests::` | PASS: all 17 creative tests, including two new adapter tests |
| Client `cargo clippy --lib --bin horizon-client --no-default-features --features native-mods --offline -- -D warnings` after creative changes | PASS |
| Narrow rustfmt for all changed Rust files; `git diff --check`; `python3 tools/check_public_tree.py` | PASS |
| `cargo build -p horizon-client --bin horizon-client --no-default-features --features native-mods --offline` | PASS |
| Real client `--offscreen --frames 2` in fresh temporary working/home/XDG/launcher paths | Window smoke BLOCKED: resources loaded, native init/metadata/shutdown logged; exit 1, `Could not find wayland compositor` |
| Full Java differential parity | NOT RUN: optional; complete Java SDK still unavailable. Existing harnesses preserved |
| Live native item/icon, old-save/native content and LAN mismatch checks | NOT RUN: persistent native content is not activated by this increment |

All executable game tests/runs used scratch `HORIZON_HOME`, launcher and XDG
paths with self-update/audio disabled; no real saves/settings were used. The
real-client attempt read privately prepared resources in place, copied no assets
into test fixtures and published no resource files. Unit fixtures are synthetic.
No source-mixin hook body, WASM runtime, item storage, save or packet code changed.
The binary regression suites include the upstream source-mixin hookability tests.

Limits / next smallest step: the existing screen is wired and asset-free draw/
click/back/scroll paths are exercised, but no actual rendered window has passed.
The linked example pack still contains Hello only; two independent mods are
covered in runtime/projection and presentation tests, not a two-mod window run.
`Creative::append_tabs` has no production caller yet. It preserves existing tab
positions and merges search, rejects ID collisions and special tabs atomically,
and explicitly rejects overflow of the fixed 2x7 layout. Native title-key/icon
resolution and pagination remain pending. Pass one frozen snapshot through
resource startup/reload, validate templates and IDs using the existing resource
architecture, then add the sample native item/tab with storage/render/mismatch
acceptance. Mutable native configuration needs a public API contract.
**Phase 2 gate remains OPEN. Phase 3 and Create were not started.**


## Previous increment — commit policy and phase 2 list/details data adapter

Root `AGENTS.md` now mandates one completed engineering step per Conventional
Commit, explicit staging, preservation of existing changes/history, relevant
pre-commit checks, commit/test reporting and no push without explicit instruction.
The outer repository was initialized; the first commit `b848dee` contains only
`AGENTS.md`. `7051696` adds private-resource/build exclusions. At the user's
explicit approval, `37a5799` records the pre-existing standalone sources and notes
as a separately identified baseline, not work authored in this increment.

New implementation commits: `8dbce9e` (complete compiled catalog and regression
test) and `58006aa` (independent UI list/details model and tests). Public host API:
`CompiledMod`, `Runtime::compiled_mods()`, `Runtime::host_side()` and the
`horizon-native-ui` read-only view/detail/dependency types. The mod-facing
`horizon-native-api` is unchanged. The UI model includes all compiled sides,
metadata, exact dependency status and static-loading/rebuild/trust notices. It
does not expose toggles or mutable settings. Server-only runtimes return no view.

| Check actually run in this increment | Result |
|---|---|
| `cargo fmt --manifest-path native-mods/Cargo.toml --all --check` | PASS |
| `cargo test --manifest-path native-mods/Cargo.toml --workspace --locked --offline` | PASS: 18 tests; 14 baseline + 4 new |
| `cargo check --manifest-path native-mods/Cargo.toml --workspace --all-targets --locked --offline` | PASS |
| `cargo clippy --manifest-path native-mods/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings` | PASS |
| Cargo metadata dependency assertions | PASS: API std-only; hello → API; runtime → API; UI → API/runtime |
| Horizon normal dependency tree without native feature | PASS: no `horizon-native-*` crates |
| Horizon `cargo test -p horizon-client --bin horizon-client --no-default-features --locked --offline` | PASS: 153 passed, 3 ignored |
| Same Horizon command with `--features native-mods` | PASS: 156 passed, 3 ignored |
| Horizon `cargo check -p horizon-client --bin horizon-client --no-default-features --features native-mods,mods --locked --offline` | PASS |
| Root staged/working diff whitespace checks; Horizon diff whitespace check | PASS |
| Horizon `python3 tools/check_public_tree.py` | PASS |

Horizon checks used fresh temporary game/launcher/XDG homes and launcher scratch
paths, with self-update/audio/Discord/telemetry disabled. No game or launcher was
launched. No assets, proprietary resources or real saves were required or copied.

Exact engine changes in this increment: **none**. All pre-existing Horizon
changes remain uncommitted and untouched. No native UI dependency was added to
Horizon. The feature-off behavior and coexistence checks above cover the existing
bridge consuming the updated runtime; they do not exercise an implemented Mods
screen. Existing WASM runtime tests and Java parity were not rerun in this
increment because no WASM, gameplay or engine code changed; earlier evidence below
is historical, not a claim of a fresh run.

Phase 2 remains OPEN: rendered list/details, mutable settings API, live resource/
creative adapters, icon resolution, old saves and network/mismatch validation are
pending. Next smallest step: generic read-only loading-model/details support in
the engine's existing Mods UI, then connect this snapshot through the client
bridge while preserving existing WASM indices and source-mixin hooks. Review the
insertion plan in `HORIZON_INTEGRATION_MAP.md` first. Create remains deferred.

## Previous increment — historical scope and evidence

Phase 0 audit and phase 1 code are implemented. The phase 1 gate is **CLOSED / PASS**
against the mandatory matrix below under the revised acceptance policy. Phase 2 has started with standalone item/tab
registration and freeze; its UI/content gate remains OPEN. Phases 3–5 and Create
have not started.

Local toolchain: rustc 1.99.1 (58cf8f9c3 2026-10-13), Cargo's selected toolchain,
no upstream toolchain override file. At that earlier increment the outer directory
had no Git repository; Horizon was independently versioned. Initial user file
`horizon_official_server.txt` was preserved. No commits, PRs, or asset uploads were
made in that earlier increment. See the latest increment above for current commits.

## Validation decision — 2026-10-30

At the user's explicit direction, this independent Rust Minecraft implementation
no longer requires absolute Minecraft Java equivalence before native-loader work.
Full vanilla differential parity and Java oracle suites are **OPTIONAL** diagnostics.
A missing JDK 25, SDK `ct.sym`, oracle assets or compositor does not block phase 1's
startup-only loader slice. No JDK was installed or requested for this revision.
Upstream parity harnesses and contribution instructions remain intact; this is the
outer integration project's acceptance policy, not an upstream policy rewrite.

Existing Horizon regressions remain blockers, including regressions found by an
optional harness. UI/world/save/network validation is required when those paths are
introduced; phase 1 does not claim those unimplemented capabilities work.

## Mandatory matrix — this revision

| Criterion | Evidence / result |
|---|---|
| Existing Horizon behavior; feature disabled | PASS: `cargo test -p horizon-client --bin horizon-client --no-default-features --offline`: 153 passed, 3 ignored |
| Native feature enabled | PASS: same client binary test command with `--features native-mods`: 156 passed, 3 ignored, rerun after descriptor API changes |
| Native + WASM feature coexistence | PASS: `cargo check -p horizon-client --bin horizon-client --no-default-features --features native-mods,mods --offline`, rerun after API changes; WASM/source-mixin production sources unchanged |
| Existing WASM loader regression suite | PASS: `cargo test -p horizon-mods --features runtime,pack --locked --offline`: 22 unit + 4 API reference + 32 real sandbox tests; 1 performance test ignored |
| Native lifecycle/dependencies/errors | PASS: native workspace tests, 14 total; missing/optional/version/API/side/cycle/duplicate errors, fail-stop state and reverse partial-startup cleanup |
| Registration/mod interaction through public API | PASS: two test mods register descriptors, read the complete frozen snapshot during setup, resolve declared cross-mod references; both input orders and Client/Server/Combined hosts |
| Rejected registration | PASS: duplicate item/tab IDs, foreign/reserved namespace, blank translation key, missing icon/member and undeclared dependency fail before setup with no partial snapshot |
| HelloWorld metadata/init/shutdown | PASS: pack tests on all host sides and `list_mods` example; earlier isolated real-client startup log below |
| API/engine isolation | PASS: Cargo metadata: API is std-only, hello mod depends only on API, runtime depends only on API; feature-off normal client graph contains no native crates |
| Formatting/static checks | PASS: native workspace `cargo check --workspace --all-targets --offline`, Clippy with `-D warnings`, `cargo fmt --all --check`; Horizon `git diff --check` and `python3 tools/check_public_tree.py` |

All Horizon test commands ran with fresh temporary HORIZON_HOME,
HORIZON_LAUNCHER_HOME, XDG_DATA_HOME, XDG_CONFIG_HOME and scratch launcher
trash/clipboard/desktop paths; self-update/audio/Discord/telemetry disabled. No
player resources, settings or saves were modified. No live game was launched in
this revision.

WASM test environment: the initial offline run passed 26 unit/reference tests but
could not build guest mods (uncached `wit-bindgen`; wasm target absent). Downloaded
only locked Rust dependencies and installed `wasm32-unknown-unknown`, built the
upstream test guests, then reran the complete suite successfully offline. The
initial environmental failures are not counted as passing tests.

## Optional diagnostics — historical results

- Full Java differential suite: NOT RUN in this revision; no JDK 25 requirement.
- Earlier supplemental game-loop comparison: 6,791 cases / 84,873 lines matched;
  compiler adaptation and scope are retained below. This is not full vanilla parity.
- Rendered real-client smoke: NOT PASSED in the earlier attempt (no compositor).
  This does not block the startup-only phase 1 gate and does not validate phase 2 UI.

## Earlier checks (historical; not all rerun in this revision)

Commands below run from the outer workspace unless explicitly prefixed `cd horizon`.

| Command / check | Result |
|---|---|
| Initial `cargo test --manifest-path native-mods/Cargo.toml --workspace` | PASS, supplied baseline 6 tests |
| Native workspace tests after changes (`--workspace --offline`) | PASS, 11 tests plus doc-test targets |
| `cargo check --manifest-path native-mods/Cargo.toml --workspace --all-targets --offline` | PASS |
| `cargo clippy --manifest-path native-mods/Cargo.toml --workspace --all-targets --offline -- -D warnings` | PASS |
| `cargo fmt --manifest-path native-mods/Cargo.toml --all --check` | PASS |
| `cargo run --manifest-path native-mods/Cargo.toml -p horizon-native-pack --example list_mods --offline` | PASS; metadata, init and shutdown actions printed |
| `cd horizon; cargo check -p horizon-client --bin horizon-client --no-default-features --offline` | PASS before bridge implementation |
| Same client check with `--features native-mods` | PASS after integration |
| `cd horizon; cargo build -p horizon-client --bin horizon-client --no-default-features --features native-mods --offline` | PASS |
| Full game binary tests with native feature in isolated environment | PASS: 156 passed, 3 ignored (including 3 generic bridge tests) |
| Full game binary tests without native feature in isolated environment | PASS: 153 passed, 3 ignored |
| Client bin Clippy with `--no-default-features --features native-mods --offline -- -D warnings` | PASS |
| `cargo tree -p horizon-client --no-default-features --offline -e normal --prefix none` | PASS: no `horizon-native-*` in normal dependency graph |
| Narrow `rustfmt --check --edition 2024 --config skip_children=true` for game.rs / native_api.rs | PASS |
| `cd horizon; git diff --check` | PASS |
| `cd horizon; python3 tools/check_public_tree.py` | PASS |
| `cd horizon; python3 tools/parity.py game_loop --list` | Selected game_loop |
| `cd horizon; python3 tools/parity.py game_loop` | BLOCKED before comparison: missing authenticated `.local/minecraft/26.3/manifest.json`; runner and its retry both failed |
| Native + WASM combined check (`--no-default-features --features native-mods,mods`) | PASS after authorized Cargo dependency download; initial offline/DNS failures resolved |
| Supplemental `PATH=/tmp/horizon-native-tools:$PATH python3 tools/test_game_loop_parity.py` | PASS: 6,791 cases / 84,873 output lines match pinned vanilla; wrapper uses current Java 25 source/target instead of SDK `--release 25` |
| `gh pr list --state open --search 'native mod' --limit 10` in Horizon | Unavailable: could not connect to api.github.com |

The three ignored upstream tests are not reported as passing. The unchanged
source-mixin hookability tests are part of the full binary suite above.
The unit tests do not establish vanilla differential parity; that comparison is optional under the decision above.

## Earlier isolated real-client attempt (supplementary)

Built native-feature executable launched with `--offscreen --frames 2`, a small
window/render distance, a fresh temporary working directory, and read-only prepared
resources from the player's private resource root. No resources were copied into
examples or fixtures. Scratch `HORIZON_HOME`, `HORIZON_LAUNCHER_HOME`,
`XDG_DATA_HOME`, `XDG_CONFIG_HOME` and launcher trash/clipboard/desktop paths were
set; self-update/audio/Discord/telemetry were disabled. Temporary logs and saves
were removed with the scratch directory.

Resources loaded; the actual client log contained:

```text
[native:hello_mod] HelloMod initialized through the public API
Native mod: Hello from native Rust 0.1.0 (hello_mod)
[native:hello_mod] HelloMod shutdown through the public API
```

Process exit 1: winit could not find a Wayland compositor. This verifies real
startup metadata and shutdown on a loop-start error, but NOT successful rendering,
world load, gameplay, persistence, integrated-server execution or a UI smoke test.
No dedicated-server world was launched. Dedicated-side selection is covered by
asset-free bridge/runtime tests and the headless build only.

## Public API / engine review

Phase 1 API: validated `ModId`, `ModMetadata` with API/license/side/exact dependencies,
registration-only `LoadContext`, side-specific setup, owned log queue, opt-in tick
subscriptions and reverse shutdown. The runtime has an explicit fail-stop state
machine. Registration completes before setup; partial startup always has a cleanup
path through the bridge owner. Mods have no engine dependency or Game references.

Engine changes: 5 client Cargo lines, 29 lockfile lines, 9 cfg-gated Game ownership/
startup lines, new generic bridge module (including tests). Reasons and alternatives
are in `HORIZON_INTEGRATION_MAP.md`. WASM and existing source-mixin hooks were not
modified. No tick, UI, item, save or packet engine changes were made.

Current limitations: exact dependency versions only; no pack manifest generator;
no persisted content registry; no game tick dispatch, mod list/settings, gameplay
or networking integration. Phase 2 now adds descriptor freeze alongside subscription
freeze; it does not yet create live engine content. Rust
source API is experimental and native code has full process privileges.

Phase 2 public API added in this revision: `ResourceId`, `ItemDescriptor`,
`CreativeTabDescriptor`, `ContentSnapshot`, registration methods on `LoadContext`,
and `ModContext::content()`. Runtime validates declarations globally before setup
and shares an immutable snapshot. Translation-key/resource resolution, rendered
icons, content persistence and network mappings are NOT yet validated.

Additional engine edits in this revision: **none**. All implementation is in the
independent native API/runtime and asset-free tests. The existing engine-change
ledger is unchanged; the next generic UI insertion is planned in the map.

Next smallest step: implement read-only compiled-mod list/details with explicit
static-link semantics, then the validated resource/creative adapter and actual UI
checks. Phase 2 stays OPEN until its own gate passes. Create remains deferred until
the base Mod API and its prerequisite gates are complete.

## Historical optional parity environment investigation

The original missing-manifest error was resolved without modifying player resources:
upstream import/verification helpers imported the authenticated client/server jars,
manifest and server dependencies into ignored `.local/`. Library paths in the private
installation differ from the manifest; remaining libraries were matched by their
original manifest SHA-1 against local jars, then imported with `lab.import_copy`.
All required client library hashes matched. No game files were downloaded or tracked.

The runtime Java 25 includes `jdk.compiler`, so a temporary `/tmp` javac wrapper can
invoke `java -m jdk.compiler/com.sun.tools.javac.Main`. However the stock command
`javac --release 25` fails with `release version 25 not supported` because this runtime
lacks the complete SDK (`ct.sym`). A temporary JDK download attempt timed out and its
partial archive was removed. A supplementary differential attempt uses current Java
25 with `--source 25 --target 25`; this compiler adaptation is explicitly distinguished
from an unmodified SDK validation. No harness or engine source was patched.

`python3 tools/parity.py --changed-files /tmp/horizon-native-changed.txt --list`
conservatively selected 358 of 360 harnesses for game.rs plus the new bridge file.
That full suite has NOT been run or declared passing. Several real-client harnesses
need a working display/compositor, which the isolated launch could not access.

Final supplemental differential result: **6,791 cases match, 84,873 output lines**. This is evidence for the tested no-native probe, not a claim of full game/world/UI parity or a modded-world acceptance pass.
