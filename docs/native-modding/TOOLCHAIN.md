# Native pack authoring and locked builds

Developer prerequisites: Python 3.11+ (stdlib tomllib), Git and the exact Rust/Cargo
version in native-modpack.toml. Follow Horizon's own host build prerequisites.
Python tools are development tools; the game and native mods remain Rust.

Run from the outer workspace, with the independent engine checkout at `horizon/`.
Use the reviewed engine commit from native-modpack.lock. Do not point tools at
player homes. No tool prepares, copies or downloads Minecraft resources.

## Recipe

`native-modpack.toml` schema 1 has required `schema`, `api`, `rust` keys and an
optional `[[mods]]` array (absence/empty array produces a zero-mod pack).
Each entry requires `package`, workspace-relative `path`, exact `version`.
Optional `feature` makes that crate opt-in through a pack feature; optional
`features = ["name"]` selects Cargo features in the mod. Implicit default mod features are
disabled; the feature list remains explicit. Omit `feature` for always-linked mods. Paths must be ordinary
reviewed source directories below native-mods/mods; symlinks/traversal, floating
versions, duplicate packages/paths and unknown keys fail. Remote/git source
fetching and registry-installed mods are deliberately unsupported: vendor trusted
mod source locally. Third-party Rust libraries may use exact =x.y.z registry
requirements plus the Cargo.lock checksums; in-tree libraries must stay within
the native workspace. Declared mod libraries belong under mods/ too.

The generator sorts packages, preserves the `create_runtime/create_for_side`
contract, emits exact Cargo version constraints and conditional vector entries.
Runtime separately validates actual mod IDs, exact metadata dependencies, sides,
API epochs and content collisions, then orders by dependency/ID. TOML declaration
order never sets gameplay order. No constructor executes during generation.

```sh
python3 tools/native_modpack.py --help
python3 tools/native_modpack.py generate
python3 tools/native_modpack.py check
python3 tools/check_native_boundaries.py --feature-off
python3 tools/native_pack_release.py verify
# Intentional reviewed updates only:
python3 tools/native_pack_release.py lock
```

`--manifest PATH` on the generator supports alternative local recipes; generated
output always goes to the same pack. Copy a chosen recipe to native-modpack.toml
before locking/releasing. Do not hand-edit generated pack files. Examples:
default Hello; `test-pack` enables Companion; `gameplay-example` enables Workshop.
Their simultaneous Creative categories may exceed the unchanged host capacity;
use each acceptance feature separately for game runs.

## Verified host builds

```sh
python3 tools/native_pack_release.py build --variant vanilla
python3 tools/native_pack_release.py build --variant native
python3 tools/native_pack_release.py build --variant native --features gameplay-example
# --release selects the release profile; default is dev.
```

Builds verify hashes/toolchain/engine revision, run the graph guard, and use
`--locked --offline --no-default-features`. They compile both horizon-client and
horizon-ctl for the current host, with no audio/controllers. General Cargo builds
can enable these host features separately; those are outside this wrapper's
receipt. Dependencies must already be cached: use `cargo fetch --locked` for each
workspace when network access is intended, then rerun offline. Failed offline
resolution is an error, never a fallback to floating versions or another engine.

The source receipt pins native tracked files, generator/build/graph tools, recipe,
both Cargo.lock files, complete rustc/Cargo identities, host triple and Horizon
commit. Dirty tracked engine source and untracked native source outside target/
are refused. Read/review source before explicitly staging new files for `lock`.
It is a reproducible **source/dependency selection** contract. Bit-identical Rust
executables across paths, operating systems, linkers, environment/build scripts or
compiler versions are not promised. Cargo configuration/environment still applies;
release builders must review it. The binary hash receipt in
horizon/target/{debug,release}/native-build-{vanilla,native}.json records the actual
command/host/features and SHA-256 of both executables. The shared Cargo output
paths are replaced by the next variant; retain wanted binaries outside source
before switching variants. A new host/toolchain needs a deliberate lock review.

## Source recipe distribution

```sh
# Commit reviewed source and the lock first; output must not already exist.
python3 tools/native_pack_release.py package /tmp/native-source-recipe.zip
```

The deterministic ZIP contains only tracked native workspace sources (including
original example art), native docs/templates, native tooling/tests, workflow,
manifest and lock. BUILD.json names the engine revision/toolchain/host and trust
notice; SHA256SUMS covers each other member. ZIP member order/time/mode are fixed.
It contains **no engine checkout, binary, player resources/saves, dependency cache
or Mojang assets**. It is a source recipe, not a .hmod or complete offline SDK.
Unknown source rights still require human review; path/hash guards cannot identify
copyright ownership from bytes. Proprietary data must never enter tracked source.

Recipients verify archive checksums, review source and obtain the exact Horizon
revision separately (in a sibling horizon/ checkout). Because verification uses
Git's tracked-file inventory, unpack into a clean integration Git checkout or
initialize a new Git repository and explicitly track only the recipe's sources.
Install the pinned toolchain, fetch locked Rust dependencies, then run `verify`
and the checks below. Prepare game resources separately from a licensed player
installation; supply them read-only only for actual scratch game runs.

## Author template and tests

Copy templates/basic/ into native-mods/mods/example/, rename its package/metadata
ID, authors and licence, and add the crate to native-mods/Cargo.toml members and
the pack recipe. Its sole dependency is the public API. The template test uses
only API contexts, proves idle registration and partial-startup cleanup. Check it
independently without editing the real pack:

```sh
python3 tools/native_author_template_smoke.py
python3 tools/native_generated_smoke.py
python3 tools/native_package_smoke.py # after committing reviewed release sources
python3 -m unittest discover -s tests -v
cargo test --manifest-path native-mods/Cargo.toml --workspace --all-features --locked --offline
cargo clippy --manifest-path native-mods/Cargo.toml --workspace --all-targets --all-features --locked --offline -- -D warnings
```

Hello/Companion demonstrate original static content, localization and independent
namespaces. Workshop demonstrates authoritative commands/recipes/entities using
only API snapshots/actions. Read their code and pack/tests/gameplay.rs for a host
integration template. Do not add runtime or engine dependencies to mod crates,
even for their dev tests; keep host integration tests in the composition crate.
Use synthetic original resources for unit tests, never private prepared files.

Test dependency/collision/failure cleanup, all host sides, no idle callbacks,
ordering under reversed composition, freeze before setup, bounded actions,
save/restart and matching/mismatching multiplayer when relevant. Real gameplay
harness: tools/native_phase3_smoke.py; always isolated temporary homes, with
read-only resources passed explicitly. Dedicated EULA acceptance requires the
human's existing explicit authorization. Sequential renderer tests are the
regression suite; preserve the documented parallel-GPU baseline SIGSEGV.

## CI

.github/workflows/native.yml has native tooling/API checks plus vanilla, native
Workshop and combined WASM/native Horizon jobs. It uses no Mojang data. The engine
is checked out at the exact source-lock revision from repository variable
NATIVE_HORIZON_REPOSITORY (default horizon-rust/horizon). Integration commits must
be accessible there; CI refuses a missing revision instead of using upstream main.
No push is performed by these tools. Workflow execution needs a hosted checkout;
local validation does not imply a hosted green run. Asset-dependent game scenarios
remain separately recorded manual/scratch acceptance.
