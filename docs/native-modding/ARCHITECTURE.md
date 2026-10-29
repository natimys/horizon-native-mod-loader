# Native, compile-time Horizon mods — architecture contract

Status: **experimental metadata/lifecycle, embedded resources and real static items
integrated**. Version 5 / 2026-10-30.

The opt-in native path now supplies Horizon's existing Content/resource loaders and
Creative UI. Registration freezes before resources load; the same registered
instance performs setup, with a common immutable snapshot shared by client/server
resource consumers. Each Game still owns its own lifecycle. Mods depend only on
the std-only public API. No native gameplay tick hooks or Create port exist.

Resource/model/template/translation failures stop activation. Save IDs remain
namespaced strings; protocol mapping uses the existing registry adapter and an
exact static-content KnownPack check before play. Native settings, dynamic/special
item models, creative pagination and registry remapping remain unsupported. See
native-mods/README.md for the concrete public contract, VALIDATION.md for actual
checks and HORIZON_INTEGRATION_MAP.md for every engine insertion. Phase 2 gate is
CLOSED / PASS for read-only UI and static-item content; no phase 3 work has begun. Full Java differential parity is
optional under ROADMAP.md; existing Horizon regressions remain blockers.

## Existing Horizon — verify against local checkout

Upstream currently documents a sandboxed WASM loader (`crates/horizon-mods`, `crates/horizon-mod-sdk`) and an options Mods UI. Separately, `horizon-mixins` offers source transformations at build time. This project intentionally proposes a third, compile-time Rust **public-API** path, not a source patcher. Before implementing, inspect the actual local Horizon version; do not assume upstream main and the checkout agree.

References:
- https://github.com/horizon-rust/horizon/blob/main/AGENTS.md
- https://github.com/horizon-rust/horizon/blob/main/docs/mods.md
- https://github.com/horizon-rust/horizon-mixins
- https://github.com/horizon-rust/horizon-native-mod-template

## Core invariants

- Native crates compile and link into the executable; a rebuild is required to add/remove code mods. No WASM, dynamic libraries, runtime Rust compilation, engine AST patching, or source mixins in this path.
- Generic engine bridge points only: mod behavior lives entirely in mod crates. The runtime schedules callbacks, enforces logical lifecycle rules, and converts typed actions into engine operations.
- `native-api` owns stable public mod-facing data types. Engine-private `Game`, internal ECS/wgpu types, raw NBT implementation, renderer handles, private storage and networking objects never cross the API boundary.
- Zero-mod integration must remain an opt-in/feature-gated path and preserve existing Horizon behavior and tests. Full Minecraft Java equivalence is not a native-loader acceptance condition. Existing WASM support must not be removed.
- A native mod is a fully trusted Rust dependency. Compilation provides memory-safety advantages for safe Rust, but does NOT sandbox malicious or unsafe code or guarantee runtime correctness.

## Dependency graph (acyclic)

```text
Horizon engine bridge -> horizon-native-pack -> example-mod
                              |                     |
                              v                     v
                    horizon-native-runtime -> horizon-native-api
```

`native-api`: traits and mod metadata; versioned namespaced identifiers; description types for blocks, items, tabs, commands, menu extensions; read-only event snapshots; allowed game actions. No Horizon-engine dependency.

`native-runtime`: registers compiled descriptors, resolves dependencies, sorts load order, validates collisions, freezes registries, dispatches events to subscribers, and provides UI-friendly immutable metadata.

`native-pack`: deterministic, explicitly generated composition root: references the chosen mod crates and calls their public registration constructors. No magic discovery during runtime. The initial implementation can hardcode one example mod, later generate this file from an ordered manifest at build time. Avoid global mutable registries and accidental link-order-dependent registration.

`Horizon native bridge`: thin generic integration at the verified initialization, UI, client tick and server tick boundaries. The bridge exposes snapshots / validated actions instead of leaking mutable engine references. Add only the adapters that the first working mod actually requires.

Do not force these names onto upstream until verifying its workspace structure; module naming can adapt while invariants stay fixed.

## Mod manifest & dependency checks

Every compiled mod provides immutable metadata:

- `id` as a lowercase namespaced-safe identifier (`example_mod`, not user-facing display name)
- `name`, `version`, `description`, `authors`, `license`
- `side`: `common`, `client`, or `server` (with clear meaning for runtime hooks)
- `dependencies` (required and optional, with version constraints)
- declared capabilities/hooks to reduce dispatch cost

The compiled mod list is authoritative. Mod discovery from a filesystem folder is NOT a feature of static linking. For a future build tool, resolve `native-modpack.toml` -> dependency graph -> generated pack Rust source/Cargo dependencies -> rebuild.

Reject duplicate mod IDs, missing required dependencies, cycles, incompatible API versions and duplicate registry identifiers with actionable diagnostics. Deterministic ordering: dependency DAG first, stable ID tie-breaker. The static-item bridge now negotiates a deterministic content/registry digest before play; broader gameplay synchronization remains future work.

## Lifecycle

Proposed stages, subject to local bridge audit:

1. `Discover/Resolve`: compile-time mod list and metadata validated.
2. `Register`: declare items/blocks/tabs/commands and callbacks via typed registrars. NO world data or UI renderer access.
3. `Freeze`: validate identifiers, produce immutable registration snapshot and runtime lookup maps; derive stable save/network mapping.
4. `CommonSetup`: shared game services ready (no player/world yet).
5. `ClientSetup` / `ServerSetup`: side-specific services ready.
6. `WorldLoad`, `WorldUnload`, `PlayerJoin`, `PlayerLeave`: scoped events.
7. `Tick` and input/action/render events: opt-in subscriptions; no unnecessary per-tick work for inactive hooks.
8. `Shutdown`: release mod-owned state.

Event handlers receive small read-only snapshots and/or restricted command buffers. The runtime validates action requests at the bridge. Avoid exposing `&mut Game` or returning arbitrary engine callbacks that bypass lifecycle constraints. Never hold references to transient engine state across calls.

## Public API areas — staged, not all at once

- Core: `Mod`, `ModInfo`, `ModId`, `ResourceId`, dependency resolution, startup/shutdown, log, event subscription.
- Commands: registration + typed args/permission model (reuse upstream parser through a bridge, not reimplement).
- Content: item and block descriptors, registries and tags; recipes; block entities; entities. Each requires save, render, inventory, server and packet review before production use.
- UI: `CreativeTab` descriptor (id, title, icon item, item filter/supplier); mod list based on compiled metadata; mod details/config screens; extension points for independent screens and menu categories. Generic UI renderer owns widgets, mods provide data/actions.
- Game: tick, interaction, inventory change, world load, block placement/mining, networking events; client and server sides must stay separated.
- Resources: namespaced asset lookup/overrides and generated resource indexes, with no bundled Mojang files.
- Later Create prerequisites: block entities, rotational energy networks, moving contraptions, recipes, fluids, sided IO, capabilities/components, client animations/particles and dedicated-server sync. Do not design a one-off Create hook instead of generic contracts.

## Creative inventory and Mods screen

Creative tabs are registry entries, NOT enum arms hardcoded for each mod. In the first UI milestone, the renderer consumes a sorted immutable list of tab descriptors, resolves icon and namespaced item handles, and filters all registered items by tab. Include search and multiple tabs; empty tabs should not break layout.

The Mods screen should consume the compiled-mod registry and show name, id, version, authors, description, dependency status, API version and side. It is not a claim that these mods can be toggled without rebuilding. If the existing WASM list is reused, display each mod's loading model (`native` versus `WASM`) and separate runtime enable/disable semantics.

## Registries & persistence

Never treat an ephemeral in-process integer registry ID as a save-file identity. Define canonical `namespace:path` identifiers and deterministic mapping to existing engine IDs (if an integer map exists). Freeze before loading a world. Store/restore by stable IDs or add explicit world-migration mapping. Network handshake must agree on registry contents; missing server-required content fails gracefully instead of corrupting a save. Do not ship new items/blocks until save/load, recipes, creative inventory, block/item render, server sync, and ID collision tests pass.

## Required acceptance tests

1. No native mods: existing Horizon baseline behavior and regression tests remain unchanged. Full vanilla differential parity is supplementary; keep upstream harnesses available and report their results separately.
2. One small native HelloWorld mod: visible in Mods UI, appropriate lifecycle events, no private engine dependency.
3. Two mods: deterministic order; missing dep, cycle, duplicate id and duplicate content id fail with clear errors.
4. UI: arbitrary new tabs/items appear without a mod-specific renderer branch; client-only UI hooks never execute on dedicated server.
5. On-world-load: registry freeze, save/reload identity, old worlds unaffected, modset mismatch handled intentionally.
6. Performance: no idle polling every frame for mods that registered no hooks; record before/after build and runtime cost.
7. Existing WASM mods and vanilla assets flow remain functional.

## Non-goals of the initial loader

- Porting Create, Fabric, NeoForge, Forge or arbitrary Java `.jar` files.
- Arbitrary mod installation into an already built executable.
- Hot reload of native code, compatibility with binary-only proprietary mods, or isolation of untrusted code.
- Stable cross-version Rust ABI. The versioned PUBLIC API is source-level and rebuilt with the game.

## Phase 2 descriptor and resource contract (implemented)

`ResourceId` validates canonical `namespace:path` names. `LoadContext` accepts
`ItemDescriptor` and `CreativeTabDescriptor` declarations only during registration.
Runtime checks ownership (namespace equals mod ID; `minecraft` is reserved), duplicate IDs per registry and
translation keys, then resolves tab icons/members after ALL mods register. Cross-mod
references require an explicitly declared active dependency. Items and tabs have
separate ID spaces; maps iterate by stable names, while tab members keep declared
display order. Empty tabs are permitted. Failed registration/freeze publishes no
partial snapshot and never runs setup; normal fail-stop cleanup still applies.

`ModContext::content()` provides a shared immutable snapshot before any setup hook,
so dependent mods can query each other's declarations without private engine types.
The item template is a symbolic host-resource reference, validated by the current
engine adapter against existing vanilla components. Embedded resources are owned
bytes submitted through LoadContext::register_resource. The host reuses Content
and mod_content, validates model/texture/translation dependencies, then feeds
Creative::load through its generic additions file. Save IDs remain namespaced
strings; matching static protocol mappings are confirmed before play. Actual
single-mod rendered/persistence evidence and the two-producer synthetic tests are
recorded in VALIDATION.md, with remaining acceptance gaps stated separately.

## Phase 2 compiled catalog and presentation contract (implemented)

Runtime captures `CompiledMod` entries for the complete validated dependency graph
before host-side filtering. `metadata()` still returns only lifecycle-selected mods;
`compiled_mods()` returns all entries in deterministic dependency/ID order, with a
`selected_for_host` flag. Catalog reads never invoke constructors or callbacks.
Selection does not assert successful setup and does not permit runtime disabling.

The std-only `horizon-native-ui` crate depends on API/runtime, never on engine
internals; mod crates do not depend on it. `ModListView::for_runtime` produces a
read-only client snapshot or `None` on a server-only host. Entries retain every
metadata field and exact dependency requirements, distinguish present dependencies
from absent optional ones, and support stable-ID details lookup. The snapshot
records whole-runtime state without inventing per-mod running/error information.
Static native loading, rebuild requirements and full process privileges have
explicit presentation notices. No toggle or settings callback is exposed.

This data adapter supplies the actual Horizon screen through a generic bridge.
Settings require a separate public configuration contract; the static item slice
does not introduce mutable native settings or gameplay callbacks.

## Phase 2 Horizon presentation / creative adapter

The host maps `ModListView` into `ModList::catalog` records and the existing
Options UI renders their list/details. Its mutable `installed` collection, WASM
owner indices and source-mixin toggle/settings path are unchanged. Font-wrapped
details show metadata, exact dependency availability, host selection and the
explicitly runtime-wide state. Catalog refresh never constructs mods, runs hooks
or persists enable/disable lines. Native configuration requires a future public
contract; there is no fabricated Settings button. Server hosts produce no view.

`Creative::append_tabs` is the generic host-side category/search boundary used by
native content through creative-additions.json. It sorts by stable ID, allocates
vacant positions in the existing 2x7 layout, preserves existing positions/search
components, and rejects duplicates/special tabs/overflow before mutation. Empty
categories are allowed. The native resource adapter resolves item/icon resources
and title keys before exposing the resulting root. Pagination remains future work.
