# Native source API and upgrade policy

The native API is an experimental Rust **source** contract. API epoch 1 is
`horizon_native_api::API_VERSION`; the API crate's source version is currently
0.1.0. Neither number is the WASM/WIT API version. There is no stable Rust ABI,
shared-library protocol, plugin directory or runtime replacement of linked code.

## Compatibility rules

- Runtime requires exact API epoch equality before any registration/setup. The
  recipe independently pins that epoch. Metadata/dependency/side validation covers
  the whole compiled graph before host selection, including optional dependencies
  that are present. Runtime metadata dependencies use exact strings, not semver.
- Source-compatible API additions may keep the epoch and increment the Cargo
  package version. A removed/renamed type/member, changed ownership, new required
  trait method, added required public struct field, changed event order, action
  error semantics or side/lifecycle behavior requires a new epoch. During 0.x,
  increment the minor Cargo version for such breaks. Do not quietly change old
  semantics in a patch. Changelogs must name the migration and tests.
- A new epoch is deliberately refused by an old host; simultaneous Rust API
  epochs are unsupported. Rebuild API, runtime, UI, every mod, static pack and
  Horizon together. Mods should use API_VERSION and record their tested API
  epoch/source version in their own release notes; successful compilation alone
  does not establish persistence or multiplayer compatibility.
- Constructor contract is `pub fn create() -> Box<dyn NativeMod>`. Constructors
  return owned instances; catalog reads do not reconstruct them. Content freezes
  before setup/world entry. Shutdown must tolerate partial setup and is idempotent
  at runtime; abort/panic=abort cannot guarantee cleanup.
- Persistent ID names are permanent `namespace:path` identities. Keep content
  schemas stable when changing mod/package versions. Removing registered content
  or changing block-entity schema versions is refused on world entry until an
  explicit migration exists. Never renumber saves or remove the world manifest
  to bypass the guard. Automatic schema migration is unsupported.
- Server/Combined execute authoritative gameplay; Client does not. Persistent
  declarations require Common. The exact content/resource/registry identity must
  match across multiplayer peers; no automatic native-code download/remapping.
  A matching recipe receipt helps reproduce builds but does not replace the
  engine's existing handshake or prove arbitrary native code deterministic.

## Deliberate upgrade procedure

1. Back up worlds outside the source repository. Read API/mod change notes and
   review trusted source, including Rust dependencies/build scripts. Never run
   `lock` merely to silence a verification failure.
2. Update the recipe's exact crate/toolchain versions, reviewed local mod source,
   and API epoch if required. Update runtime metadata dependencies separately.
   Change the pinned Horizon revision only after its integration tests pass.
3. Run `generate`. For intentional composition changes, allow Cargo to update
   both affected lockfiles once with the normal targeted native/Horizon check
   commands **without** `--locked` (keep `--offline` if caches suffice). Review the
   lock diffs; do not regenerate the entire engine lock just to add a local mod.
4. Run author and integration tests, all generated compositions, off/on regression,
   WASM coexistence, saves, matching/mismatching peers and client/server scenarios
   relevant to the change. Use scratch homes for game runs. Commit only reviewed
   source. Run `lock`, inspect/commit its hashes and run `verify`.
5. Rebuild the custom executable for each OS/architecture. Distribute the same
   reviewed recipe/content to client/server administrators. Changes are source
   upgrades and rebuilds, never a promise to load old native binaries.

## Trust and scope

Native mods and their Rust build scripts execute with full process/user privileges.
They can read files, access network, start processes, block or crash the game.
They are **not sandboxed**; the dependency graph checker is an architectural
regression guard, not a security scanner. Do not build/install unknown code.
Metadata licences and authors are declarations, not automatic provenance checks.
WASM continues to use its separate loader and sandbox policy.

Create, energy/fluids/kinetics, dynamic models, arbitrary entity NBT/inventories,
mutable runtime settings and automatic world migration remain outside phase 4.
