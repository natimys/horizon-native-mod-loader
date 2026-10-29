# Compile-time native loader — experimental phase 3 gameplay

Trusted Rust crates are statically linked into a custom Horizon executable.
Adding or removing a mod requires rebuilding. There is no binary plugin ABI,
filesystem discovery, hot reload or sandbox. Never compile untrusted mods.
The independent workspace uses only std; mod crates depend only on the public API.

- `api`: metadata, validated `ModId`, exact-version dependencies, sides,
  registration-only `LoadContext`, owned log actions and lifecycle callbacks;
  canonical `ResourceId`, frozen item/tab/block/recipe/entity/command descriptors;
  owned gameplay snapshots and deferred actions.
- `runtime`: deterministic dependency DAG / ID order, API/version/side checks,
  registration then freeze then setup, opt-in tick subscribers, reverse cleanup;
  complete compiled catalog, including side-inactive mods.
- `ui`: engine-independent read-only mod list/details snapshots for client hosts;
  metadata, dependency presence and explicit static-link/trusted-code notices.
- `mods/hello`: API-only common mod; an original token item, creative tab, embedded
  model/texture, English/Russian strings and init/shutdown logs.
- `pack`: explicit composition root, the only production crate naming concrete mods.
  Optional `test-pack` adds the independent API-only Companion acceptance mod;
  `gameplay-example` adds the API-only Workshop gameplay example.

```sh
cargo test --manifest-path native-mods/Cargo.toml --workspace --offline
cargo clippy --manifest-path native-mods/Cargo.toml --workspace --all-targets --offline -- -D warnings
cargo run --manifest-path native-mods/Cargo.toml -p horizon-native-pack --example list_mods --offline
cargo check --manifest-path horizon/Cargo.toml -p horizon-client --bin horizon-client --no-default-features --features native-mods --offline
```

The two-mod acceptance composition is opt-in and needs no engine changes:

```sh
cargo test --manifest-path native-mods/Cargo.toml --workspace --all-features --offline
cargo build --manifest-path horizon/Cargo.toml -p horizon-client --bin horizon-client --no-default-features --features native-mods,horizon-native-pack/test-pack --offline
```

HelloWorld and Companion own different namespaces, textures and EN/RU strings.
The default pack still contains only HelloWorld. The two acceptance features may
exceed the existing Creative free positions when combined; the adapter refuses
overflow atomically, and this phase does not expand that capacity. Adding the earlier-sorted Companion
changes transient protocol numbers; it must preserve HelloWorld's saved string ID.

Omitting `native-mods` omits the native crates from the client dependency graph.
The native feature is independent of the existing WASM `mods` feature and upstream
source-mixin hooks. Do not use those hooks as this API or depend on private engine crates.
Keep the outer directory structure: the client uses optional relative path dependencies;
a standalone upstream checkout cannot build the native feature without these crates.

Each `Game` owns a runtime. A threaded window selects Client, integrated/dedicated
servers select Server, and an in-frame window selects Combined. Common mods run
once per Game, so threaded singleplayer has two common instances; they share no state.
The bridge freezes registration before resource loading, retains that same instance
for Game setup, and cleans up on normal/error return. Integrated servers register
their own instances and must reproduce the shared frozen resource snapshot.
Shutdown logs errors while still attempting every registered mod in reverse order.
A process abort/exit or panic=abort cannot guarantee cleanup. A native panic is not sandboxed.

Runtime registration completes for all active mods before setup starts. All registered
mods, including ones whose setup did not run, must tolerate shutdown after partial startup.
Initialization and failed ticks are terminal; failed-batch actions are discarded.
Shutdown keeps diagnostic actions even when cleanup reports errors and is idempotent. Optional dependencies, when present, constrain order and exact
version just like required dependencies. An active mod cannot depend on an inactive
side-specific mod. Whole-graph metadata validation precedes side selection.

The legacy standalone `subscribe_tick`/`tick` API remains unwired. Gameplay uses
`subscribe_gameplay` and `NativeMod::gameplay` with owned authoritative snapshots.
Phase 2 now freezes item/tab descriptors after every mod registers and before setup.
IDs belong to the declaring mod namespace; duplicate IDs, missing native tab icons/
members and cross-mod references without declared dependencies fail before setup.
Callbacks read the shared immutable snapshot through `ModContext::content()`.
`LoadContext::register_resource(path, bytes)` embeds original resource files in the
binary under `assets/<mod-id>/...` or `data/<mod-id>/...`. Foreign/reserved paths,
traversal, duplicate files and hidden data-driven item declarations fail at freeze.
Persistent items, tabs and resources require a Common mod. `Runtime::freeze()`
performs registration without setup; `initialize()` accepts that frozen runtime
without invoking register again. Registration-owned state survives resource loading. The original into_descriptors
item/tab extraction signature is retained; resource-aware hosts use the additive
into_declarations contract.

The opt-in host adapter materializes embedded files in the game's cache, converts
item templates to the existing Content reader, and uses Horizon's mod_content
pipeline for components, commands, protocol registries, models and texture discovery.
It validates vanilla templates, collisions, owner-provided English strings, static
model leaves/parents/textures and PNG decoding before opening worlds. Arbitrary
translation keys are aliased to existing item display-name keys per language.
Creative additions feed the existing Creative::append_tabs layout/search adapter;
ordinary pack reload reuses the frozen snapshot without constructors or callbacks.
There is no second live item registry or separate texture/localization pipeline.

Initial items use a vanilla component template and a plain static model leaf.
Dynamic/special model trees, gameplay behaviors and native settings are not exposed.
Creative categories use vacant slots in the existing 2x7 layout; overflow fails
atomically. Pagination is not implemented. Resource/model errors stop startup;
native content is never silently dropped in a fallback to vanilla resources.

Saves keep Horizon's existing string item IDs. Protocol IDs are transient and
assigned through the existing sorted content adapter. A content identity covering
native declarations/resources and the resulting built-in registry mapping is
negotiated through KnownPack before play (including memory joins and
reconfiguration). Mismatched/absent static content is refused; no download or
network remapping is provided. Empty native packs preserve the original root.
Existing WASM content overlays retain their namespaces and IDs.

Two API-only test producers exercise this whole adapter with original synthetic
resources, opposite composition orders, localized names, icons, inventories and
conflicting/missing declarations. A real single-HelloWorld client has additionally
passed rendered Creative/Mods/details and process-restart persistence checks.
See VALIDATION.md for the exact evidence and remaining phase 2 gate criteria.

`Runtime::metadata()` lists only mods selected for the host's lifecycle.
`Runtime::compiled_mods()` lists the complete validated pack in dependency/ID order;
`selected_for_host` describes side selection, not runtime enablement or successful
setup. Catalog reads use captured metadata and never call mod code.
`horizon_native_ui::ModListView::for_runtime(&runtime)` captures that catalog and
the runtime's state, or returns `None` for server-only hosts. Details are looked up
by stable mod ID and retain all metadata and declared dependency versions. Required
dependencies were validated by resolution; absent optional ones are shown as such.
Snapshots remain unchanged when runtime state changes; capture again to refresh.
The model has no toggle or mutable settings actions. It is now connected to
Horizon's existing Options > Mods screen through the optional engine bridge.
The test sandbox cannot connect to the Wayland socket; an isolated launch outside
the sandbox can use the session compositor. Normal visible-window and simultaneous
two-mod rendering acceptance remain separately recorded in VALIDATION.md.

Mandatory gates protect existing Horizon behavior, native on/off, WASM coexistence
and API/lifecycle correctness. Full Minecraft Java parity is optional; JDK 25 is not
a prerequisite solely for it. See ROADMAP.md and VALIDATION.md in
`docs/native-modding/` for the explicit decision and actual results.


## Phase 3 public gameplay contract

Build the optional example (default composition remains HelloWorld):

```sh
cargo build --manifest-path horizon/Cargo.toml -p horizon-client \
  --bin horizon-client --bin horizon-ctl --no-default-features \
  --features native-mods,horizon-native-pack/gameplay-example --locked --offline
```

Workshop registers `workshop:counter`, its block item/tab, a shapeless recipe
(one stick + one cobblestone), a versioned `visits` field and `/workshop:place`.
The command queues a counter two blocks east of its player. Once per second the
mod increments the field through a deferred action. Player joins and committed
break/place events produce messages. All behavior is in `mods/workshop`, whose
only dependency is native-api. Nothing in Horizon imports Workshop or tests its ID.

Registration remains separate from running worlds. `register_block` reuses a
vanilla template's static states/facts/shapes; `register_recipe` initially accepts
1..9 explicit item ingredients, shapeless, with a bounded output stack.
`register_command` initially accepts owned namespaced zero-argument commands.
`register_block_entity` attaches a unique owned schema to a native single-state
block, version >=1, at most 64 named signed i64 fields. Persistent declarations
require Common mods, producing identical client/server catalogs. Unsupported
schema versions are refused; automatic migrations, inventories and custom entity
renderers are not provided.

`GameplaySubscriptions` opts into players, world, committed player blocks and
block-entity ticks. Runtime callbacks run in dependency/ID order, only on Server
or Combined hosts, excluding Client mods. A threaded client never simulates a
second copy. Per dimension: UUID-sorted departures then joins, queued player
break/place events in capture order, WorldTick, entity ticks sorted by position,
then action application. Paused/frozen world ticks do not tick entities. World
snapshots include UUID-sorted players (possibly in other dimensions). There are
no borrowed engine/world/player handles and no cancellation of vanilla actions.
Explosions/pistons/mod writes do not synthesize player block events.

`GameplayContext` permits Tell, SetBlock and SetBlockEntity. Queues are owned;
callbacks cannot forge their runtime owner. Each callback is capped at 256 actions,
messages at 4096 bytes; the host bounds pending events/actions to 4096. Callback
or queue errors discard their event's actions and stop gameplay dispatch for the
runtime. Host validation rejects a whole queued batch before applying writes;
engine-operation failures are logged individually, with no rollback guarantee.
Block writes use ordinary checked placement, never load chunks, and wait for the
selected dimension's next pass. Entity writes require the declaring owner, schema
keys/version, a matching loaded block/entity, and are rechecked after earlier
queued writes. Player/checked command/action placement initializes entity data;
world generation of native block entities is outside this first contract.

Typed entity data lives in ordinary chunk block_entities and uses the existing
chunk/update packets. A registered single-state block replacement removes its
payload immediately. Names, entity schemas, recipes and resource bytes determine
the exact static pack identity; no registry downloading or numeric remapping is
provided. Matching builds use identical mappings. Incompatible peers disconnect
before play. World manifests store only required item/block names and entity
schema versions; checks run before migrations/chunk decoding, and recording uses
the ordinary session lock. Additive packs are accepted. Removing declared content
or changing a schema version requires migration before opening that world, even
if that registered content was never placed. Never remove mods to bypass this
check. Native-disabled vanilla worlds without a manifest follow the normal path.

Run real isolated acceptance with the supplied resource root:

```sh
python3 tools/native_phase3_smoke.py --binary horizon/target/debug/horizon-client \
  --ctl horizon/target/debug/horizon-ctl --resources "$HORIZON_TEST_RESOURCES" \
  --mode in-frame --reload
# Also run --mode integrated (includes LAN late join).
# Dedicated requires the human's existing EULA acceptance:
# --mode dedicated --eula-already-accepted
python3 tools/native_static_pack_probe.py --port SCRATCH_SERVER_PORT
```

The scripts create only temporary game/launcher/XDG homes, disable self-updates
and audio, and retain display runtime access. Logs/saves stay in the reported
scratch paths. Do not commit those caches, screenshots, resources or saves.
See `docs/native-modding/PHASE3_VALIDATION.md` for actual outcomes and limitations.
Mutable native settings, Creative expansion, dynamic models and Create remain out
of scope. Source API is experimental; native crates are fully trusted process code.
