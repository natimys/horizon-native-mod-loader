# Phase 3 — gameplay extension acceptance (2026-10-30)

Gate: **CLOSED / PASS** for the bounded phase 3 contracts below. Actual client,
in-frame, LAN and dedicated gameplay/save/sync scenarios passed. The inherited
parallel-GPU baseline failure is distinguished below from the passing serial
regression suite. Create has not started.

## Scope and public contracts

- `BlockDescriptor/register_block`: owned namespaced blocks and their items,
  resolved through existing vanilla templates/content tables.
- `RecipeDescriptor/register_recipe`: shapeless recipes with 1..9 explicit items,
  checked dependencies, output ID and stack limits, ordinary Recipes execution.
- `CommandDescriptor/register_command`: owned namespaced zero-argument commands,
  owner-only server dispatch, existing command report/client transport.
- `BlockEntityDescriptor/register_block_entity`: one schema per owned single-state
  native block; version >=1, up to 64 signed integer fields. Existing typed chunk
  storage/save/update packets carry the generic extension payload.
- `GameplaySubscriptions`, owned `PlayerSnapshot`, `WorldSnapshot`,
  `BlockEntitySnapshot`, `GameplayEvent`, `GameplayContext` and owner-tagged
  `QueuedAction`: joins/leaves, world ticks, committed player block events, entity
  ticks; Tell, SetBlock, SetBlockEntity. No Game/world/renderer references.
- Optional external `horizon-native-pack/gameplay-example` composes Workshop.
  Its only dependency is native-api. It declares the counter/recipe/tab/command,
  and owns all event reactions and counter behavior. Default pack stays HelloWorld.

No settings, Creative pagination, dynamic model API, arbitrary NBT, custom entity
renderer/inventory, capability/machine system, modpack generator or Create work.
The API is experimental source-level Rust; native code is trusted process code,
not sandboxed or loadable into an arbitrary prebuilt executable.

## Exact engine boundary

All engine edits and alternatives are in HORIZON_INTEGRATION_MAP.md. Summary:

| Entry | Why an engine adapter is necessary |
|---|---|
| `native_content::{validate,resource_root}` | Convert frozen descriptors to existing Content/Recipes/command/type tables, resolve names/translations, include all declarations in static identity. No second loader. |
| `game/native_api`, new `game/native_gameplay` | Own authoritative snapshots/queues and validate actions; mods cannot borrow private Game. |
| `game.rs::tick_world`, successful player break/place callsites, `finish_close_world` | Observe actual committed operations and authoritative dimension ticks; clear session queues. Existing WASM/mixin hooks stay independent. |
| `game/{chat,lan_actions,commands}` command entrypoints | Route namespaced player/guest/owner-console calls; block actions use existing checked `place_state`. |
| `game/commands::place_state` | Initialize registered data after checked writes. |
| render crate `chunk/block_entity`, `containers`, `world::World::set`; Game container matches | Generic data variant, empty inventory, immediate removal on block replacement. No custom renderer algorithm. |
| `save::{encode_block_entity,decode_block_entity}` | Stable type/version/fields in ordinary chunk block_entities. No sidecar entity store or numeric save IDs. |
| `lan/chunks::{ChunkEncoder::new,entity_tag}`, `game/remote::remote_block_entity_data` | Registered generic entity types through existing initial/update packets; support live placements before an old entity exists. No packet-layout/protocol rewrite. |
| `registry_pack::{check_world,record_world}`, `WorldRequest::prepare` | Check required persistent names/schema versions before migrations/chunks; record under ordinary world session lock. Prevent missing-content save loss, including native-disabled builds. |
| `agent::block_json` | Inspect actual client-received generic payload, analogous to existing sign inspection; sync acceptance does not query the server instead. |

Production Horizon code imports no concrete example mod, tests no concrete mod ID
and holds no mod-specific Game field. Mods/runtime/API dependency direction is
acyclic. Native feature off has no native crates in its normal dependency graph.
Original WASM runtime, source-mixin native_hooks.rs and parity harnesses are intact.

## Checks actually run

Commands ran offline with the selected rustc 1.99.1/Cargo 1.99.1. Game/server runs
used fresh temporary game/launcher/XDG homes and launcher trash/clipboard/desktop,
self-update/audio/Discord/telemetry disabled. Display runtime was preserved.
Private prepared resources were read only; never committed or used as unit fixtures.
Dedicated EULA acceptance was explicitly confirmed by the human before launch.

| Check | Actual result |
|---|---|
| Native workspace `cargo test --workspace --all-features --locked --offline` | PASS: 29 tests; includes lifecycle/dependency/side/failure/collision tests, blocks, recipes, commands, entity schemas, two independent callback producers and optional Workshop composition. |
| Native all-target/all-feature Clippy `-D warnings`, fmt check | PASS. |
| Horizon client lib/bin, native feature off | PASS: 669 lib + 153 bin; 21 lib + 3 bin ignored. |
| Horizon client lib/bin, native + gameplay-example | PASS: 677 lib + 160 bin; 21 lib + 3 bin ignored. |
| Final affected `native_content::tests` after cache correction | PASS: 8; includes repeated overlay load with identical tables/digest. |
| Generic extension codec/update-type tests | PASS: 2. |
| Generic replacement/removal test | PASS: 1; break+replace in one tick cannot resurrect payload. |
| World requirements + static identity tests | PASS: 2; additions accepted; missing content/schema change refused without overwriting manifest. |
| Combined `native-mods,mods,gameplay-example` binary check | PASS. |
| Existing WASM runtime/pack suite | PASS: 22 unit + 4 API reference + 32 real sandbox tests; 1 performance test ignored. |
| Horizon native lib/bin Clippy `-D warnings` | PASS, repeated after final cache/initializer corrections. |
| Renderer suite with `-- --test-threads=1` | PASS: 526, 15 ignored. |
| Renderer suite with default parallel test threads | FAIL: SIGSEGV among GPU tests, including when no smoke client remained. |
| Same default-parallel renderer suite on original phase 2 `fd7316fd1`, in a fresh Git-tracked-source-only /tmp copy | Same SIGSEGV; 540 baseline tests selected, no new extension test. Establishes inherited baseline failure, not a phase 3 regression. No unrelated GPU code change attempted. |
| Dependency graph assertions | PASS: native-api std-only; all three example crates and runtime depend only on API; feature-off graph contains no native crates. |
| Narrow rustfmt, both diff whitespace checks, public-tree guard | PASS; public-tree guard repeated in Horizon at delivery. |
| Final headless gameplay-example client + horizon-ctl build, locked/offline | PASS. |
| Full Java differential parity | NOT RUN; optional under ROADMAP acceptance policy. Existing harnesses preserved. |

## Real scenarios already completed

Actual offscreen frames used the NVIDIA Vulkan renderer, not mocked rendering.

| Scenario | Evidence |
|---|---|
| Integrated server | Player chat command placed a counter; client received type/version/data and increasing visits; no client gameplay callbacks. |
| Actual crafting | UI 2x2 slots 100/101 held one stick/cobblestone, result slot 200 showed workshop:counter; taking it consumed ingredients and placed one native item in hotbar slot 2. |
| Player block events | Real break/place actions emitted Workshop broke/placed messages; removal erased data and replacement started a new counter. |
| Integrated save/restart | Existing scratch counter resumed with visits 149 after value 107 before shutdown; stable block/entity names retained. |
| LAN late join and live updates | Independent TCP guest received the existing counter, increasing payload, and used its own command. Host and guest both observed a new counter at the guest's coordinates; decode_errors=0. |
| LAN static mismatch | Local bounded protocol probes with absent and different identity both received configuration Disconnect before play. |
| Dedicated | Real loopback headless server + matching client: command placed workshop:counter at [12,-60,9], client payload grew; server had no renderer/client setup. |
| Dedicated save/restart | Console save-all/stop, same world restarted, reconnecting client read counter payload visits 129 (>=2 before stop). Final console stop saved normally. |
| Dedicated static mismatch | Same absent/different identity probes refused before play. Initial whitelist refusal was corrected in test configuration; it was not counted as a pack-check pass. |
| In-frame | Actual crafting, block actions/callbacks, data increment and normal save passed. |
| Native-disabled world refusal | Real feature-off binary refused the native in-frame save with explicit World content mismatch; SHA-256 of all 22 save files remained unchanged. No chunk migration/decode occurred. |
| Repeatable in-frame harness after cache fix | PASS: fresh scratch platform, real crafting/events, same-home restart, block/entity data restored. |
| F3+T and existing Mods UI | PASS: actual resource reload retained counter payload (visits 123), no repeated HelloMod initialize; both HelloWorld and Workshop rows remained in the existing Mods page. |

Repeatable integrated/LAN and dedicated harnesses both returned PASS after the
cache correction. The integrated harness crafted through UI, preserved the
registered entity through same-home restart and verified guest late join/live
updates; the dedicated harness placed/ticked/transmitted data and stopped cleanly. `tools/native_phase3_smoke.py --help` gives the parameters;
`tools/native_static_pack_probe.py --port PORT` probes only loopback. Tools create
scratch directories and retain logs/saves there; they bundle no player resources.

## Resolved failures and limits

- Full binary compilation found two missing exhaustive non-container arms after
  library codec tests passed. Fixed in its own commit; final off/on bin suites pass.
- Owner-console commands initially bypassed the native player dispatcher; player
  chat worked. Fixed the generic entrypoint/acting UUID; real console smoke passed.
- Same-home automated restart found duplicate type insertion in the cached
  overlay. Fixed by deriving types from immutable base input; regression test and
  actual repeatable same-home in-frame run pass. The gate was not closed on the
  earlier one-off startup tests alone.
- The first harness broke at variable terrain/transition timing. The harness now
  builds an open platform with vanilla commands and waits for the UI transition;
  no engine workaround was added for the test.
- Boundary review also found namespace-agnostic vanilla container suffix
  fallback. The registered schema now takes priority for fresh placements; a
  regression test exercises the real fallback and preserves a live non-default
  extension value; final native binary suite is 160 passed, 3 ignored.
- An initial frame-limited missing-content probe exited before asynchronous world
  entry. The unrestricted probe reached the real refusal and verified file hashes.
- The optional reload harness initially waited for a pipelines-ready log line.
  Reload reuses pipelines, so that was an invalid success criterion. The helper
  now observes renderer recreation, unchanged init count, actual payload and Mods
  rows; that helper passed in the existing scratch world.
- Parallel GPU tests have an independently reproduced phase 2 baseline SIGSEGV;
  the serial suite is the passing regression result. Full vanilla Java equality
  and visible-window mouse operation are not claimed.
- World requirements conservatively retain every registered persistent ID. Adding
  content is compatible; removing content or changing schema version is refused
  until an explicit migration exists. Custom entity worldgen, multi-state entity
  blocks, inventories/renderers, arbitrary data and ID downloading/remapping are
  unsupported. Native commands are initially player-owned, zero-argument commands.
- Callback/queue failure stops gameplay dispatch; host validation rejects a whole
  queued batch, engine writes report individual failures without rollback. Reads
  use owned snapshots; writes never force-load chunks. Cross-dimension queues wait
  for that dimension's tick. Ordering/limits are in native-mods/README.md.

## Atomic commits

Outer repository (implementation and integration notes):

- d46c5f3 docs: phase 3 audit/plan.
- 1823e90 feat(native-api): block descriptors/freeze.
- 634d469 feat(native-api): shapeless recipes.
- 1a0aea2 feat(native-api): authoritative snapshots/action queues.
- eacf2b6 feat(native-api): namespaced commands.
- 77be4f9 feat(native-api): versioned block entities.
- eafe776 feat(native-example): optional Workshop composition/tests.
- 9ad817f test(native): repeatable real gameplay/save/LAN/dedicated tooling
  and bounded local static-pack mismatch probe.
- Final architecture/validation documentation is a separate commit; the final
  response reports the inclusive range.

Horizon repository:

- 83281aeae blocks through existing tables.
- 1e39734ce recipes through existing crafting.
- 2957fc75a authoritative snapshot/action bridge.
- 43f94a065 existing command/report routes.
- 35e52305f generic extension storage/codec/transport.
- 29914d025 non-container UI exhaustive arms.
- 0152521b2 native entity schema/tick/action adapter.
- cb0a88318 owner-console route and client payload inspection.
- 3516980ec stable world-content requirements.
- c5a9d676c idempotent cached entity mapping.
- f84396c94 registered entity schema wins over suffix-based template fallback;
  existing live payload remains unchanged (new regression test).

No push. Pre-existing untracked user files are excluded from every commit.
Next smallest work after acceptance is phase 4 toolchain/API-versioning work;
Create remains deferred to its later explicit roadmap phase.
