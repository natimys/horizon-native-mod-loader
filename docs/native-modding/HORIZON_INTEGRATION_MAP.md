# Horizon integration map

Audited 2026-10-30 against local checkout (observations, not assumptions).
Initial status: only untracked `horizon_official_server.txt`; preserve it.
`gh pr list --state open --search 'native mod'` could not reach api.github.com.

## Observed ownership and callsites

- `horizon/Cargo.toml`: resolver 2, edition 2024; client/render/items/world/protocol are separate crates. Native crates remain in the independent outer `native-mods` workspace, using optional path dependencies from the client.
- `crates/horizon-client/src/main.rs::main` installs logging and calls `game::run`.
- `src/game.rs::run() -> Result<()>`: Options::read, dedicated preflight, synchronous resource loading or `run_loading`, then `start_game` / `run_event_loop`.
- `src/game.rs::Game::new(...) -> Result<Self>` constructs each game before world opening. `Game` owns resource Arcs and private state. `src/game/execution_role.rs::ExecutionRole` fixes Client / IntegratedServer / DedicatedServer at construction. Connecting does not change that role.
- `src/game/integrated.rs::run_integrated` runs a second Game, starts WASM mods, and ticks the authoritative server. A client with server_thread=false simulates in-frame. Do not mistake the window for the integrated server.
- `src/game/levels.rs::tick_server(paused)`: iterates dimensions; only the first dimension calls `Game::tick`, others call `tick_world`. An authoritative tick hook must run once per server tick, not once per dimension.
- `src/game.rs::tick`, `tick_world`, `open_world`, `close_world`; `src/game/dedicated.rs::run_dedicated`: headless loop and world lifecycle. `src/game/saving.rs` / `persist.rs` serialize world and entities.
- `src/game/mods.rs::content_root`, `start_mods`, `list_mods`, `tick_mods`; `src/game/server_mods.rs::tick_server_mods`: independent existing WASM path. `mods` feature controls wasmtime; content overlays work without it.
- `src/game/native_hooks.rs`: EXISTING source-mixin hooks, NATIVE_HOOKS_API=2, including `list_native_mods`. Not the new public Rust API. Preserve these and their hookable bodies.
- `crates/horizon-client-render/src/mod_list.rs::{Installed,NativePage,ModList}`: existing metadata/UI model assumes toggles and mixin-native settings. Reusing `native: Some` for permanently linked API mods would incorrectly offer a runtime switch. A read-only loading-model field/row adapter is needed in phase 2.
- `crates/horizon-client-render/src/options_ui.rs`: `Page::Mods`, `native_rows`, `Act::ToggleMod`, `Page::NativeMod`. Engine screen routing lives in `src/game/options.rs`; client-render owns presentation.
- `crates/horizon-client/src/creative.rs::{Creative,Tab}`: data-driven Vec<Tab>, `load` / `from_json` read creative-inventory.json, search index built there; fixed top/column presentation still needs pagination review. `Names::load` merges namespaced language files. Adding tabs by enum/match is unnecessary.
- `crates/horizon-items/src/items.rs::{Items,ItemStack}`: string item names (`Arc<str>`), `Items::load` / `catalog` consume resource JSON. `item_nbt.rs` writes the string `id`; never persist transient numeric IDs.
- `crates/horizon-client/src/mod_content.rs::{build,resource_root}`: overlays use ContentMod descriptors, clone template entries into item/block/model/creative tables. Reuse through an adapter after validation; no copying prepared assets into the mod workspace or test fixtures.
- `crates/horizon-protocol/src/packets/{configuration,play}.rs`, client `src/lan/`, `src/game/remote.rs`: packet boundaries. Existing WASM content sync does not prove compatibility for new native content; handshake and mismatch tests are required before introducing persistent native items.

Paths starting `src/` above are relative to `horizon/crates/horizon-client/`.

## Phase plan and engine-change ledger

Phase 0: map and plan only. Phase 1: API/runtime/pack tests first, then feature-gated Game-owned bridge. Phase 2 follows only after phase 1 checks; new persistent content requires save/render/network acceptance. Phases 3–5 depend on those gates. No Create implementation.

| File | Entry point | Reason | Alternative considered | Required test |
|---|---|---|---|---|
| `crates/horizon-client/Cargo.toml` | optional `native-mods` feature | explicit static pack/runtime linking; default path untouched | filesystem discovery cannot load statically linked Rust | Cargo metadata default/native; client checks |
| `crates/horizon-client/src/game.rs` | module, cfg field, Game::new | own generic runtime per execution role; freeze before world creation | process singleton fails integrated-server lifetime/side separation | default/native headless check, role tests, lifecycle tests |
| `crates/horizon-client/src/game/native_api.rs` (new) | bridge owner initialization and Drop | translate fixed role, drain log actions, reverse shutdown on exit/errors | private Game pointer/global callbacks forbidden; existing mixin hooks are unrelated | pure runtime lifecycle tests; bridge role mapping tests |

The phase 1 rows above are implemented. `Cargo.lock` gained only the four native package records and three client dependency names (29 lines). `game.rs` gained nine feature-gated lines: module, runtime owner, startup before any temporary World construction, and field initialization. The new `native_api.rs` translates roles and logs actions, owns cleanup via Drop, and contains asset-free bridge tests. The field is intentionally `_native_api`: ownership is its only phase 1 purpose. No tick or gameplay engine insertion is authorized by the first slice; APIs without a validated host remain unavailable.

## Open acceptance constraints

- Initially no imported parity artifacts; authenticated local oracle imports are now in ignored `.local/`. Player resources remain read-only. Never use them as public/unit-test fixtures or publish them. SDK `ct.sym` and a working compositor remain unavailable; full Java parity is optional under the 2026-10-30 decision in ROADMAP.md. Neither prevents this startup-only phase 1 gate; actual UI/world validation is still required when those paths change.
- Native content requires audited overlay adaptation, old-save tests and LAN handshake/mismatch validation. Standalone descriptors alone do not meet the phase 2 gate.
- No server UI calls: metadata adapter may consume immutable metadata on the client only.

## Review of implemented slice

- No mod-specific imports, identities or callbacks in production engine code. The engine chooses only HostSide; composition remains in the outer pack.
- No edits to WASM, source-mixin hooks, item/block storage, save or network behavior. Tick API remains standalone until an authoritative hook is validated.
- Generic runtime field is cfg-elided with the feature off. `cargo tree --no-default-features -e normal` confirms no native crates in that graph.
- Lifecycle contract and partial-startup cleanup are documented in `native-mods/README.md`; each Game has an independent common instance.
- Phase 1 gate is CLOSED / PASS against the mandatory Horizon regression/API matrix in `ROADMAP.md`. Java SDK/display availability does not block this startup-only slice. Supplemental compiler-adapted comparison remains optional evidence only. See `VALIDATION.md`; phase 2 is started but its UI/content gate remains OPEN.

## Phase 2 first slice — no additional engine insertions

Plan: implement item/tab descriptors, canonical names, ownership/collision/dependency
checks and an immutable freeze snapshot in the independent API/runtime. Validate two
API-only test mods on client/server/combined hosts before writing a presentation or
resource adapter. This slice needs no engine edit: `LoadContext` and `ModContext`
already form the public registration/read boundary. The phase 1 engine ledger above
is unchanged.

Next proposed insertion (not implemented): read-only compiled-mod rows in
`horizon-client-render/src/mod_list.rs`, rendered by `options_ui.rs`, supplied by
`horizon-client/src/game/mods.rs` through the generic native bridge. Existing
`Installed::native` implies a switch/settings page and cannot faithfully represent
a permanently linked mod; add loading-model semantics before wiring rows. Keep
client presentation out of dedicated servers. Content overlay adaptation follows
only after template validation, creative pagination, old-save and mismatch checks.
The descriptor snapshot alone does not close the phase 2 UI/content gate.
`Runtime::metadata()` lists selected mods for one host. The standalone catalog/UI
slice below now accounts for inactive side-specific mods, without engine edits.

## Phase 2 compiled catalog/UI data slice — 2026-10-30

Observed at session start: pre-existing modifications in Horizon's `Cargo.lock`,
`crates/horizon-client/Cargo.toml`, `src/game.rs`, plus untracked
`src/game/native_api.rs` and `horizon_official_server.txt`. All are preserved;
none was edited or included in this session's commits. The outer repository was
initialized for the requested commit policy. Its authorized baseline includes
only existing native sources/integration notes, separately from new work.

Implemented outside the engine: `Runtime::compiled_mods()` retains the full
validated metadata graph before host selection, while `metadata()` keeps its
existing lifecycle semantics. `native-mods/ui` maps that catalog into client-only
read-only list/details snapshots, including exact dependency status and static
loading notices. Server-only hosts receive no view. No new engine ledger rows:
this increment needs no engine insertion and does not change the existing WASM,
source-mixin, world, storage, packet or renderer code.

Next proposed engine plan (still unimplemented):

| File / entry point | Purpose and necessity | Alternative considered | Acceptance |
|---|---|---|---|
| `horizon-client-render/src/mod_list.rs::Installed` | Generic loading-model and read-only details contract, distinct from the existing switchable `native` page | Reusing `dev` or `native` misrepresents loading/toggle semantics | Existing WASM/mixin list and persistence tests; linked rows cannot toggle |
| `horizon-client-render/src/options_ui.rs::list_rows` and details routing | Render data from arbitrary compiled entries without mod identities | Standalone UI adapter cannot draw engine-owned widgets | Two mods, empty list, details/back navigation, long/localized text and actual rendered smoke |
| `horizon-client/src/game/mods.rs::list_mods` / generic native bridge | Supply captured compiled metadata on client hosts without rerunning constructors or hooks; keep WASM indices stable | Reconstructing the pack for presentation could execute constructors twice | Feature off/on and combined WASM checks; no dedicated-server UI calls; refresh without duplication |

Do not stage pre-existing engine changes with these future insertions. Preserve
source-mixin hookability and keep a separate engine integration commit.
