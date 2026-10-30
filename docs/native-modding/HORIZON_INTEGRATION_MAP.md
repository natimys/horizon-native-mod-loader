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

## Phase 2 real item slice — audited plan, 2026-10-30

Observed: `Items::load` consumes items.json; `ItemStack` owns an Arc<str> ID,
`item_nbt` persists that string. `mod_content::Plan::{items,json}` already adapts
components, command and protocol registries; `build/resource_root` overlays
assets/data without changing the resource decoders. `Content::read` accepts an
ordinary directory and validates namespaces/limits. Native embedded files can be
materialized into an isolated, content-addressed cache and fed to this SAME path.
No second engine registry or texture decoder is necessary. `Creative::append_tabs`
already validates placement and search; a generic additions resource can feed it.
`lang::Language` and `creative::Names` already discover overlay namespaces.

Implementation steps (each independently committed):
1. Split runtime registration/freeze from setup; add owned embedded resource
   declarations with namespace/path/duplicate checks. Persistent content belongs
   to common mods so client/server cannot silently produce different catalogs.
2. Generic native resource adapter translates frozen descriptors to existing
   Content directories, validates required item models/textures/translations,
   calls mod_content, and writes a creative additions resource. Synthetic tests
   use original generated fixtures only.
3. Prepare/freeze the first runtime before resource loading; Resources owns that
   prepared instance and snapshot. Game takes it for setup; integrated servers
   register their own instances and must match the resource snapshot. Pack reload
   reuses the snapshot. No extra mod constructor for resource discovery.
4. HelloWorld declares an original item, model, texture and English/Russian strings
   using only the public API. Test real Items/Creative/NBT/protocol mappings.
5. Validate feature matrix, WASM and isolated runtime; investigate display access;
   document mandatory GUI checks honestly and leave phase 2 open if unavailable.

| Candidate engine file / entry point | Reason | Alternative considered | Test |
|---|---|---|---|
| `horizon-client/src/native_content.rs`, `lib.rs` | Optional generic descriptor-to-Content adapter using existing overlay | Separate registry/resource pipeline duplicates engine contracts | Synthetic two mods, template/resource/collision errors, stable mappings, NBT |
| `horizon-client/src/creative.rs::load` | Read generic additive tabs resource through append_tabs | Writing a second creative UI duplicates placement/search | Existing creative tests plus additions/search/localization |
| `horizon-client/src/game/native_api.rs` | Freeze before resources; own prepared runtime and shared snapshot | Discovery runtime plus Game runtime loses register-owned state | Registration once, side comparison, cleanup |
| `horizon-client/src/game.rs::{load_resources,Resources,Game::new,creative_tab_title}` | Transfer prepared runtime, resolve tab title key | Late initialization cannot supply resources before world loading | Feature off/on tests, synchronous/worker startup |
| `horizon-client/src/game/{startup_menu,packs}.rs` | Pass host role; reuse frozen content on reload | Reconstructing runtime on reload repeats hooks | Combined build, reload contract |

Existing untracked outer `.agents/`, `.codex/`, README-SETUP.md and upstream
horizon_official_server.txt are user-owned and excluded from commits. No remote is
configured in upstream, so `gh pr list` cannot inspect concurrent upstream PRs.
Network mismatch handling must be audited before claiming external multiplayer
compatibility; the default integrated server also uses protocol item IDs.

### Static item protocol compatibility insertion

Observed: `lan::data::SyncData::builtin` and `game::remote::join_server` load
`horizon-protocol::registry::Builtin` from registries.json. Item IDs are NOT a
synchronized dynamic registry. `net::configure` currently answers no known packs;
`lan::conn::login_and_configure` only offers vanilla core. Thus different static
content could reinterpret the same integer. A native item must require an exact
static-content identity before configuration finishes, including memory joins.

| File / entry point | Reason | Alternative considered | Test |
|---|---|---|---|
| `horizon-client/src/registry_pack.rs` (new), `lib.rs` | Generic optional static-content KnownPack read/validation contract | Native-specific packet or duplicated item ID tables unnecessary | Equal/different/absent identities and malformed marker |
| `horizon-client/src/native_content.rs::resource_root` | Emit deterministic registry/content identity using existing KnownPack mechanism | Runtime ID order alone cannot detect incompatible peers | Reversed public registration gives same identity |
| `horizon-client/src/lan/data.rs::SyncData`, `lan/conn.rs::login_and_configure` | Offer required static-content pack; refuse absent/different confirmation before play | Letting an unmatched client join risks wrong item interpretation | Synthetic handshake refusal plus matching memory join |
| `horizon-client/src/net.rs::Session/configure`, `game/remote.rs::join_server` | Pass resource identity into connection; compare server pack before play/reconfiguration | Global mutable identity breaks independent sessions | Default wrappers unchanged; token mismatch tests; feature-off suite |

This is phase 2 item identity validation, not phase 3 gameplay hooks or registry
synchronization. Peers must have matching static content; no download or numeric
remapping is offered. Empty/native-disabled roots emit no marker or requirement.

Implemented generic creative resource reader: upstream `30cc874de`, 18 creative
tests PASS. Descriptor/resource adapter: `4af7b388b`, 3 synthetic tests PASS; no
prepared/player assets in fixtures. First static-model adapter supports plain
model leaves, inheritance, generated icons and ordinary PNG textures. Dynamic or
special models fail explicitly until separately validated.

### Implemented real-item ledger — 2026-10-30

The earlier candidate rows in this section are now implemented (except no separate
native registry/resource decoder was added). Latest engine commits:
`30cc874de` creative resource reader; `4af7b388b` native resource adapter;
`f94a8a663` preparation/startup/reload ownership; `ea73e0266` protocol compatibility;
`e2d990f01` owner-specific translation validation; `ae5ac6191` public-producer and
legacy-content regression tests. All include the agent attribution trailer.

| Exact engine entry | Implemented purpose / alternative / actual check |
|---|---|
| `creative.rs::Creative::load` | Optional creative-additions.json -> from_json + append_tabs; avoids second UI/search/layout. 18 creative tests PASS. |
| `native_content.rs::resource_root`, `lib.rs` | Frozen descriptors -> ordinary Content::read directories -> existing mod_content::resource_root. Embedded data/assets go to game cache only. Plain model validation uses horizon-assets and png, not a new decoder. Five synthetic adapter tests PASS, real rendered item PASS. |
| `game/native_api.rs::{NativeResources,NativeRuntime::prepare}` | Mutex transfers prepared instance to first Game; common frozen snapshot compared for subsequent Games. Alternative discovery runtime would discard register-owned state. Five bridge tests PASS; real client/integrated setup + reverse shutdown PASS. |
| `game.rs::{Resources,load_resources,Game::new}` | Prepare before resource loading, retain ownership, setup before worlds; prevent fallback from silently removing native items. Feature off/on tests PASS. |
| `game.rs::creative_tab_title` | Resolve generic title keys using existing language lookup. EN/RU actual rendered tab PASS. |
| `game/startup_menu.rs::loader`, `game/packs.rs::stage_pack` | Carry role to worker preparation; resource reload consumes existing frozen snapshot. Compiled native/combined paths PASS; reload smoke recorded in VALIDATION. |
| `registry_pack.rs`, `lib.rs` | Generic optional content identity, required on both sides. Avoid a parallel registry-sync protocol. Identity/helper and real memory configuration mismatch tests PASS. |
| `native_content.rs::resource_root` marker | Deterministic digest of sorted declarations/files + existing registry mapping, independent of base/cache paths. Two reversed API-only producers and separate cache roots PASS. |
| `lan/data.rs::SyncData::load` | Read optional required KnownPack; no marker means existing behavior. Full library native off/on PASS. |
| `lan/conn.rs::login_and_configure` | Offer identity via existing KnownPack; reject missing confirmation before play. Matching real integrated join PASS; mismatch helper tested. External full server-login mismatch smoke NOT RUN. |
| `net.rs::{Session::connect_resources,configure,receive_play}`, `game/remote.rs::join_server` | Carry resource identity per connection and reconfiguration, reject mismatch/skipped confirmation. Existing Session wrappers remain available. Memory configuration tests and real integrated persistence PASS. |

No item storage, save codec, renderer implementation, WASM runtime or source-mixin
hooks were edited. Native model decoding/texture baking and language loading are
still the ordinary engine pipeline. Test assets are original generated pixels and
synthetic JSON; no private prepared resource files were added to either repository.
Native resource errors fail before any world is opened. Source-mixin hookability
remains covered by the binary suite.

Wayland audit: DISPLAY=:0, WAYLAND_DISPLAY=wayland-0 and the compositor socket are
present. A sandboxed Unix socket connect returns EPERM; the identical read-only
connection outside the sandbox succeeds. A scratch offscreen client outside the
sandbox successfully rendered via the actual Vulkan GPU. Thus the earlier
"Could not find wayland compositor" was a sandbox access failure, not a native
modloader regression. XDG_RUNTIME_DIR was deliberately preserved for display access.

Two-mod rendered acceptance follow-up: the optional `native-pack/test-pack` feature
adds an independent `native-mods/mods/companion` crate depending only on native-api.
It is composed entirely outside Horizon; production default remains HelloWorld.
Any Cargo.lock update is limited to the new optional package/pack dependency record,
required for locked reproducible builds. No engine feature, module or mod-specific
branch is added. The test will reopen the single-mod scratch world with Companion
sorting before HelloWorld, so the existing protocol adapter changes HelloWorld's
transient integer while its saved name stays unchanged. Results, not this plan,
determine whether the phase 2 gate can close.

### Phase 2 gate result

Two-mod acceptance passed in the real offscreen Vulkan client: both Mods rows,
both tabs/icons, two native IDs in hotbar, EN/RU names. Adding earlier-sorted
Companion moved Hello Token's protocol number 1658 -> 1659, while the actual saved
hello_mod:token/count/slot survived. Actual TCP probes for absent and wrong static
identity received configuration Disconnect before play. A separate world created
by the feature-off binary reopened in the final native binary with vanilla sticks
(count 3, slot 0) unchanged. F3+T retained content without replaying constructors.

Phase 2 gate is CLOSED / PASS under ROADMAP.md's read-only/static-content contract.
The optional test pack needed no further Horizon file or feature changes; locked
build/check commands passed and Horizon Cargo.lock remained unchanged. The two
API-only mod crates and native-api/runtime dependency graph were reviewed again.
No client hook/catalog renderer runs on server-only runtimes. Full dedicated-world
and visible-window/in-frame smoke remain NOT RUN; see VALIDATION.md for exact scope.

No phase 3 gameplay hook and no Create port was started. Mutable settings, dynamic
item model trees and creative pagination still require independent API proposals.
Final engine range for this stage: `30cc874de`–`fd7316fd1` inclusive. Native API,
examples/composition and documentation commits live in the outer repository;
VALIDATION.md records the stage ranges. Existing untracked user files are preserved.

## Phase 3 audit and bounded implementation plan — 2026-10-30

Phase 2 baseline reviewed: outer bf77a3a, Horizon fd7316fd1. Horizon working
 tree is clean; outer untracked .agents/, .codex/, README-SETUP.md and upstream
horizon_official_server.txt are pre-existing user files. `gh pr list --repo
horizon-rust/horizon --state open --search 'native mod'` failed (network access).
No upstream API is inferred from the online branch.

Observed locally:
- `mod_content::Plan`, `build`, `resource_root` already clone vanilla block
  states/facts/models/items and append protocol/command tables. Native blocks
  should translate to the existing `data/<owner>/horizon/block/` format.
- `crafting::Recipes::load/find` loads namespaced recipes from ordinary data
  overlays. The initial public recipe contract can be bounded shapeless recipes
  of explicit item IDs, using this loader and the existing crafting execution.
- `Game::tick` dispatches existing WASM server events before local simulation;
  remote windows return through `tick_remote` first. `levels::tick_server`
  selects each dimension; `as_guest` supplies the acting guest. Player block
  events already have precise post-success callsites in `break_block/use_item`.
- `server_mods.rs` is WASM-feature conditional and must retain its semantics.
  `native_hooks.rs` belongs to the separate source-mixin system: do not edit it.
- `place_state` is the existing checked authoritative write (neighbours, light,
  tracking, containers). Native actions must use it after callbacks, never raw
  world writes. `lan::Hub::set_mod_commands` already merges command reports.
- `Chunk::block_entities`, `BlockEntity`, save encode/decode and chunk/update
  packets carry typed entities. A generic extension payload can reuse these;
  no second world save or custom network protocol is required. Custom renderers,
  menus, arbitrary NBT, machine/capability systems are outside this phase.
- Static block/item/entity-type mappings are not vanilla dynamic registries.
  Keep phase 2's exact KnownPack identity requirement, extend its digest to all
  gameplay declarations; mismatches must disconnect before play.

Independently testable milestones: (1) block descriptors/overlay; (2) explicit
shapeless recipe descriptors/overlay; (3) subscribed owned player/world/block
snapshots and bounded validated action batches, authoritative execution only;
(4) namespaced zero-argument commands with existing command-tree transport;
(5) generic versioned integer-field block entities with existing chunk save and
packet transport; (6) optional API-only gameplay example, real client/in-frame/
LAN/dedicated/save/reload/mismatch and no-native/WASM checks. Each completed
milestone gets an atomic implementation commit and required tests.

Candidate engine insertion ledger (proposals until implementation is recorded):

| File / entry point | Reason | Alternative considered | Required check |
|---|---|---|---|
| `native_content.rs` | Translate block/recipe/entity descriptors and hash all registries | Separate loaders duplicate Plan/Recipes | Synthetic templates/collisions, actual crafting, stable names |
| `game/native_api.rs`, new generic gameplay bridge | Snapshot/dispatch/action ownership and role guard | Mods cannot borrow private Game; WASM/mixin hooks stay separate | Order, failures, no client authority, two producers |
| `game.rs::tick`, player block-event callsites; `levels.rs::tick_server` | Authoritative dispatch and dimension-aware actions | Public API alone cannot observe committed game operations | Integrated/in-frame/dedicated, normal block replication |
| `game/chat.rs`, `game/lan_actions.rs`, existing command-tree adapter | Route registered native commands with collision checks | Calling chat parser from a mod bypasses API | Vanilla/WASM command preservation, matching LAN tree |
| `chunk/block_entity.rs`, `save.rs` | Generic versioned extension payload in existing entity storage | Sidecar files break chunk lifetime/atomic persistence | Codec/reload/removal and vanilla tests |
| `lan/chunks.rs`, `game/lan.rs`, `game/remote.rs` | Generic registered entity-type mapping and existing packet updates | New payload protocol unnecessary | Initial chunks + live update + late join |

No mutable settings, Creative capacity expansion, dynamic models, runtime ID
remapping, Create work or changes to vanilla gameplay rules are planned.

Phase 3 milestone 1 implemented: `BlockDescriptor`, `register_block`, frozen
`ContentSnapshot::blocks`; duplicate block/item IDs and tab references validated
before setup on all three host sides. Engine edit is limited to
`native_content::{validate,resource_root}`: generate existing block declarations,
require real vanilla templates and own translations, include blocks in identity.
Synthetic table test checks unchanged vanilla IDs, appended block/item mapping,
translation alias and missing-template refusal. No world/save/renderer edit yet.

Milestone 2 implemented: `RecipeDescriptor/register_recipe`, freeze validation
(counts, duplicates, references and declared dependencies); native_content emits
ordinary shapeless recipe JSON, loads `Items::catalog` + `Recipes::load` and
refuses unknown items/oversized output before world load. Seven adapter tests
PASS, including actual `Recipes::find` on a 2x2 grid. Engine commits 83281aeae
(blocks) and 1e39734ce (recipes); no crafting implementation edits.

Milestone 3 insertion implemented in `game/native_gameplay.rs` and existing
`game/native_api.rs`: runtime-owned event/player/action queues; `game.rs`
post-success player block callsites and `tick_world` end dispatch snapshots.
`finish_close_world` clears session queues. No edits to native_hooks.rs or
server_mods.rs. World ticks pause with the simulation; player joins/leaves are
UUID-sorted before queued player-block events, then world snapshot, then actions.
Only authoritative Games dispatch. Each callback batch is fail-stop/all-or-none
on callback or public queue validation errors; host preflights current queued
states/dimensions/loaded positions before writes. Engine write failures are
reported individually (ordinary gameplay writes do not promise rollback).
Cross-dimension writes wait for that dimension's pass; bounded queues fail-stop
on overflow. Mods receive no engine objects. Native binary tests: 159 PASS,
3 upstream ignored; native gameplay runtime tests: 2 PASS. Actual scenario
acceptance is still pending; these checks alone do not close phase 3.

Milestones 4–5 implemented: commands reuse the resource command report
(`native_content::resource_root`) and existing chat/guest entrypoints; namespaced
zero-argument command dispatch is owner-only and server-only. No Hub command-tree
or WASM runtime edits were needed. Extension storage uses generic
`BlockEntity::Extension`, non-container views, save encode/decode, existing
ChunkEncoder type lookup and remote entity updates. `World::set` removes the
payload on replacement (initial entities require single-state blocks). New
registry mapping `horizon:extension_block_entity` maps block name -> registered
entity type number; both maps are covered by the required static identity.
Native API exposes versioned <=64 integer fields, owned entity snapshots and
queued writes with owner/schema/position validation. Existing `place_state` and
successful player placement initialize data; loaded typed entities retain it.
`native_gameplay` ticks entities sorted by position after world snapshots.

Exact additional engine entries / alternatives / tests:
- `containers::{id,contents,slots,slots_mut}` and Game's two container UI matches:
  generic entities carry no inventory. Alternative fake vanilla container would
  corrupt semantics. Initial library checks passed but the full binary check
  exposed two missing exhaustive UI arms; corrected separately in 29914d025.
- `World::set`: remove extension data immediately on block replacement, avoiding
  resurrection during break+place in one tick. Dedicated removal test PASS.
- `save::{encode_block_entity,decode_block_entity}`: persist stable type name,
  version and integer data within ordinary chunk block_entities. Codec test PASS.
- `lan/chunks::{ChunkEncoder::new,entity_tag}`: existing registry table and packet
  payload, avoiding a parallel protocol. Registered type/update-tag test PASS.
- `game/remote::remote_block_entity_data`: accept registered extension updates
  even when no old entity exists, after block/type mapping check. Needed for live
  placements; initial chunk uses the same existing decoder.
- `native_content::resource_root`: validate single-state blocks and append type
  mappings before digest. Synthetic native block/type table test added.
- `game/commands::place_state`: initialize registered data after checked writes;
  public action adapter cannot create engine entities without this host boundary.

These are generic storage/adapter contracts; no concrete gameplay mod is imported
by Horizon. Real save/late-join/update acceptance remains pending until recorded.

Save compatibility audit follow-up: block palettes persist names, but the existing
chunk decoder cannot retain simulation semantics for missing blocks. Phase 3's
save/reload/mismatch requirement therefore also needs a conservative world-entry
check. Proposed minimal insertion: generic required-content names/schema versions
in the existing resource identity marker; `registry_pack` reads/validates a world
manifest, and `WorldRequest::prepare` checks it before migrations/chunk reads,
records it after the existing session lock. Alternative silent missing-name
fallback risks destructive saves. No transient integers are persisted. Additions
are compatible; removing registered content or changing entity schema is refused
explicitly (migration is outside this phase). Native-disabled worlds with no
manifest keep their existing path. Tests: missing/default/additive/schema cases,
actual restart and a no-native refusal without touching the saved chunk data.

Real smoke exposed a console-only route missing native dispatch although the
player chat/network command worked: `Game::run_command` must also invoke the
owner dispatcher, respecting `lan_acting_uuid`. This serves agent/owner console
commands through the same public contract. `agent::block_json` gains generic
extension data, analogous to existing sign inspection, to verify actual client
initial/update packets without querying server state. No mod IDs in either hook.


Phase 3 real same-home restart found and fixed cached type re-insertion:
`native_content::resource_root` must derive block-entity types from the immutable
base table before appending descriptors, because `mod_content::resource_root`
may return an already populated cache. Registry mappings/digest now remain
identical over repeated calls; the synthetic adapter test exercises this exact
reuse. This is a host adapter correction, with no extra engine insertion.

Final boundary review found that the existing `BlockEntity::for_block` recognizes
some container suffixes across namespaces. `native_gameplay::initialize_entity`
now gives a freshly placed registered schema priority over that template fallback,
while retaining existing extension payloads. A bin test constructs the real
suffix fallback, verifies replacement by the registered schema, then checks that
a subsequent initialization preserves a non-default value. This adds no engine
insertion; it corrects ownership within the existing generic initializer.


## Phase 3 delivered gate

Gate CLOSED / PASS. PHASE3_VALIDATION.md contains every commit, final test counts,
real crafting/block events, integrated/in-frame/LAN/dedicated save/restart and
packet mismatch evidence, no-native world refusal, F3+T and Mods UI acceptance.
Parallel renderer GPU tests reproduce the same SIGSEGV in phase 2 fd7316fd1;
current serial renderer tests pass (526/15 ignored). Final engine stage range:
83281aeae–f84396c94 inclusive (11 atomic commits). No WASM implementation,
source-mixin native_hooks.rs, vanilla renderer algorithm or protocol packet layout
was changed. Source-only baseline comparison used a temporary Git archive,
without private resources or personal saves. All actual games used scratch homes.
Root tooling: 9ad817f; the outer stage starts at d46c5f3. No push. Next smallest
roadmap stage is phase 4; no Create work was performed.

## Phase 4 audit — ecosystem/toolchain

Observed at Horizon f84396c94: optional pack path in
`crates/horizon-client/Cargo.toml` is `../../../native-mods/pack`;
`game/native_api.rs` and `native_content.rs` consume generic runtime/catalog data.
`native-mods/pack/src/lib.rs::{create_runtime,create_for_side}` is the existing
composition boundary. `API_VERSION = 1` is independent of the API crate's 0.1.0
source version. PHASE3_VALIDATION.md records client/server/save/network acceptance.
Phase 4 needs only developer tooling around these boundaries. See PHASE4_PLAN.md.

| File / entry point | Reason | Alternative considered | Test |
|---|---|---|---|
| No engine insertion proposed | Existing path dependency and pack constructors accept generated composition | New engine settings/branches have no concrete need | Rebuild and test off/on using generated pack; graph and lifecycle checks |

## Phase 4 delivered gate

Gate CLOSED / PASS; PHASE4_VALIDATION.md records the source-recipe and generated
composition tests, locked off/on/combined-WASM builds/regressions, real integrated/
LAN/in-frame save/restart/mismatch/reload, source ZIP roundtrip and serial renderer
526/15 result. Engine remains f84396c94 with no phase 4 edits or commits.
Public API epoch/runtime/adapter/storage/protocol implementation is unchanged;
only external native-pack composition is generated. No concrete mod entered
engine code, no new Game field, no new lifecycle/gameplay hook.

The candidate insertion ledger therefore remains empty: existing pack constructors
and optional relative dependencies suffice. Source tooling, docs, original author
template and CI live in the outer repository. Hosted CI execution is NOT RUN and
requires availability of the exact engine pin; no push. The parallel GPU baseline
diagnosis is preserved, with no attempted fix. No Create work.
