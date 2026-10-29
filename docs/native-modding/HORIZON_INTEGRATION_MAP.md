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

## Phase 2 rendered catalog plan — 2026-10-30

Session baseline: the phase 1 client Cargo/lock/game/native_api changes are already
uncommitted; keep them out of phase 2 commits unless separately authorized.

Use a separate read-only catalog collection in the EXISTING `ModList`, rather
than inserting entries into `installed`: WASM screen owners and source-mixin
settings continue to use their original indices, IDs and toggle persistence.
`OptionsUi` will draw catalog buttons and a details page with its current rows,
scrolling, focus and Done/Esc navigation. The native UI adapter is the sole source
of native metadata/dependencies/state; opening the screen must not create mods.

| File / entry point | Reason | Alternative considered | Test |
|---|---|---|---|
| `horizon-client-render/src/mod_list.rs::ModList` | Generic read-only catalog records alongside switchable installed mods | Extending Installed would touch source-mixin hook bodies and conflate WASM owner indices | Toggle/serialization unchanged, empty catalog |
| `horizon-client-render/src/options_ui.rs::Page, list_rows` | Existing Mods list/details widgets for read-only catalog records | Adapter alone cannot draw the engine widgets | Two entries, scrolling/localized text, Done/Esc, WASM/mixin regression |
| `horizon-client/Cargo.toml::native-mods`, `Cargo.lock` | Optional existing native-ui adapter dependency | Duplicating runtime catalog/dependency logic in engine | Native off/on/combined feature builds |
| `horizon-client/src/game/native_api.rs::catalog` | Read-only snapshot from the existing runtime owner | Reconstructing the pack would run constructors twice | Host selection and unchanged runtime state |
| `horizon-client/src/game/options.rs::open_options_at` and new `native_catalog.rs` | Translate public UI snapshot into existing ModList records; wrap with existing font | Engine-independent adapter cannot depend on Horizon widgets/fonts | Server exclusion, public metadata mapping, refresh replaces catalog |

Creative follow-up: audit resource overlay construction and tab positioning before
inserting native descriptors. Existing `Tab` is data-driven, but `tab_position`
assumes columns 0..6; pagination/collision validation is necessary. New persistent
items remain gated by template/render/storage and multiplayer mismatch checks.

## Phase 2 catalog integration implemented

`ModList::catalog` / `CatalogMod` now carry read-only, font-wrapped records.
`OptionsUi::Page::ModDetails` shares the existing widgets, scrolling and navigation.
`Game::open_options_at` replaces catalog rows from `NativeRuntime::catalog()` using
`native_catalog::rows`; source-mixin and WASM installed vectors are untouched.
The optional native-ui Cargo dependency is cfg-elided with native-mods off.
Server runtime snapshots return None; state is explicitly runtime-wide, while
selection and dependency availability remain separate. No native settings contract
exists yet, so catalog entries expose details only.

### Next creative insertion plan

Implement a generic `creative.rs::Creative::append_tabs(Vec<Tab>)` first: consume
ordinary engine Tab data, deterministically allocate only vacant vanilla tab
positions, merge category members into existing search, reject duplicate IDs,
special-tab redefinition and capacity overflow atomically. Existing vanilla
layout and search remain untouched when no tabs are appended. This is a narrow
existing-menu adapter, needed by any resource/native tab producer; adding tabs
by editing individual engine match arms is not viable. Validate with synthetic
(non-Mojang) tab/item names, empty tabs, localization/search, collisions and no-op.

| File / entry point | Reason | Alternative considered | Test |
|---|---|---|---|---|
| `horizon-client/src/creative.rs::Creative::append_tabs` | Generic additive category/search adapter with collision-free placement | Raw `tabs.push` omits search indexing and can overlap fixed tab buttons | Deterministic two producers, atomic failures, vanilla no-op, search |

This bounded adapter must report capacity overflow; pagination is still pending.
Do not expose a new takeable item in a world yet. `mods::content_root` runs before
`load_resources` in synchronous startup, `startup_menu` worker and pack reload.
The native runtime currently freezes in Game::new AFTER those resource loads;
thus a proper native item overlay requires passing one frozen snapshot through
these existing loading paths, not constructing another runtime for resources.
`mod_content::build` appends network item IDs, and `Content` currently owns a
package source, so neither is a drop-in native descriptor adapter. New stable
items need template/icon/storage/old-save and mismatch checks before activation.

### Creative adapter implemented and remaining handoff

`Creative::append_tabs` is now implemented, with 17 creative tests passing. The
engine ledger row above describes the actual edit. It has no production caller
yet: native descriptors are NOT silently converted into takeable items, and no
world/save/network behavior changes. The first native overlay must resolve every
item template/icon before calling this boundary, and pass native title keys to
the existing language lookup. It must fail atomically on collision or capacity.

Engine commits this increment (inclusive): `1f92f3e8b`–`859e73057`.
`e79d9f260` is the explicitly authorized, separate pre-existing phase 1 baseline;
new work is `1f92f3e8b`, `74cfb34af`, `859e73057`. No changes to existing
`native_hooks.rs`, WASM loader, item storage, save files or network packets.
Actual client startup/shutdown passed with isolated paths; window creation failed
with `Could not find wayland compositor`. This does not satisfy rendered UI
acceptance. Phase 2 remains OPEN and phase 3/Create remain unstarted.
