# Compile-time native loader — experimental phase 2 foundations

Trusted Rust crates are statically linked into a custom Horizon executable.
Adding or removing a mod requires rebuilding. There is no binary plugin ABI,
filesystem discovery, hot reload or sandbox. Never compile untrusted mods.
The independent workspace uses only std; mod crates depend only on the public API.

- `api`: metadata, validated `ModId`, exact-version dependencies, sides,
  registration-only `LoadContext`, owned log actions and lifecycle callbacks;
  canonical `ResourceId`, item/tab descriptors and a read-only content snapshot.
- `runtime`: deterministic dependency DAG / ID order, API/version/side checks,
  registration then freeze then setup, opt-in tick subscribers, reverse cleanup;
  complete compiled catalog, including side-inactive mods.
- `ui`: engine-independent read-only mod list/details snapshots for client hosts;
  metadata, dependency presence and explicit static-link/trusted-code notices.
- `mods/hello`: API-only common mod; init/shutdown log example.
- `pack`: explicit composition root, the only production crate naming concrete mods.

```sh
cargo test --manifest-path native-mods/Cargo.toml --workspace --offline
cargo clippy --manifest-path native-mods/Cargo.toml --workspace --all-targets --offline -- -D warnings
cargo run --manifest-path native-mods/Cargo.toml -p horizon-native-pack --example list_mods --offline
cargo check --manifest-path horizon/Cargo.toml -p horizon-client --bin horizon-client --no-default-features --features native-mods --offline
```

Omitting `native-mods` omits the native crates from the client dependency graph.
The native feature is independent of the existing WASM `mods` feature and upstream
source-mixin hooks. Do not use those hooks as this API or depend on private engine crates.
Keep the outer directory structure: the client uses optional relative path dependencies;
a standalone upstream checkout cannot build the native feature without these crates.

Each `Game` owns a runtime. A threaded window selects Client, integrated/dedicated
servers select Server, and an in-frame window selects Combined. Common mods run
once per Game, so threaded singleplayer has two common instances; they share no state.
The bridge starts before Game creates worlds and cleans up on normal/error return.
Shutdown logs errors while still attempting every registered mod in reverse order.
A process abort/exit or panic=abort cannot guarantee cleanup. A native panic is not sandboxed.

Runtime registration completes for all active mods before setup starts. All registered
mods, including ones whose setup did not run, must tolerate shutdown after partial startup.
Initialization and failed ticks are terminal; failed-batch actions are discarded.
Shutdown keeps diagnostic actions even when cleanup reports errors and is idempotent. Optional dependencies, when present, constrain order and exact
version just like required dependencies. An active mod cannot depend on an inactive
side-specific mod. Whole-graph metadata validation precedes side selection.

The standalone runtime supports explicitly subscribed ticks, but the engine does NOT
call them yet: an authoritative client/server tick boundary belongs to phase 3.
Phase 2 now freezes item/tab descriptors after every mod registers and before setup.
IDs belong to the declaring mod namespace; duplicate IDs, missing native tab icons/
members and cross-mod references without declared dependencies fail before setup.
Callbacks read the shared immutable snapshot through `ModContext::content()`.
This is descriptor registration only: templates still need host validation, and no
items/tabs are inserted into Horizon resources, saves or networking. The Mods list/details
now use Horizon's existing UI; mutable settings and gameplay adapters remain pending.
Two API-only test mods exercise ordering,
cross-mod reads, empty tabs and rejected registrations without assets or a display.

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
Window rendering acceptance still needs a working compositor.

Mandatory gates protect existing Horizon behavior, native on/off, WASM coexistence
and API/lifecycle correctness. Full Minecraft Java parity is optional; JDK 25 is not
a prerequisite solely for it. See ROADMAP.md and VALIDATION.md in
`docs/native-modding/` for the explicit decision and actual results.
