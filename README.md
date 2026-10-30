# Horizon Native Mod Loader

An independent, experimental Rust modding project for
[Horizon](https://github.com/horizon-rust/horizon). Write mods as Rust crates,
compose a reviewed pack with Cargo, and compile a custom game or dedicated-server
binary. The engine integration lives in a separate
[Horizon fork](https://github.com/natimys/horizon-native-engine), pinned here as
the `horizon/` submodule.

**Status:** source API epoch 1, crate version 0.1.0. Metadata/lifecycle, static
content, authoritative gameplay hooks and source-pack tooling are implemented.
This is an experimental source release, with breaking changes still possible.
It is not an official Horizon release or endorsed by Horizon, Mojang or Microsoft.

## How it works

```text
Horizon bridge -> generated native pack -> your mod crate
                           |                    |
                           v                    v
                    native runtime ------> native API
```

Mods depend on `horizon-native-api`, explicitly declared mod libraries and Rust
libraries. They never import engine internals. The runtime validates metadata,
resolves dependencies deterministically, freezes content before worlds load,
and dispatches subscribed hooks. The bridge translates owned snapshots and
deferred actions into existing engine operations. The engine never names a
concrete mod. Registration and world-running lifecycle are separate.

| Path | Purpose |
| --- | --- |
| `native-mods/api` | Engine-independent, std-only public Rust contract |
| `native-mods/runtime` | Validation, ordering, lifecycle, frozen registries and events |
| `native-mods/pack` | Generated static Cargo composition |
| `native-mods/ui` | Read-only metadata presentation adapter |
| `native-mods/mods` | Hello, Companion and Workshop example crates |
| `horizon/` | Pinned engine fork, preserving upstream history and licensing |
| `tools/` | Development-only Python recipe, audit and validation tools |
| `docs/native-modding/` | Architecture, API policy, integration ledger and evidence |

## Relation to Horizon's modding systems

Horizon's existing `horizon-mods` / `horizon-mod-sdk` path runs sandboxed
WebAssembly mods. It remains independent and has coexistence regression coverage
in this project. Horizon also has source-mixin hooks. This loader adds a public
Rust API and static Cargo composition; it does not use source rewriting or mixins.

| | This native loader | Existing Horizon WASM path |
| --- | --- | --- |
| Distribution | Reviewed Rust source compiled into a custom executable | WASM mod packages |
| Trust | Full process privileges, including any dependency/build script | Horizon's WASM sandbox and host contract |
| Add/remove code | Rebuild the executable | Managed by the existing loader |
| Compatibility | Pinned source API + engine revision | Separate SDK/runtime contract |

This does not load arbitrary mods into a prebuilt game. Java/Fabric/NeoForge jars,
`dlopen` plugins, a stable binary ABI and hot loading are not supported.

## Implemented capabilities and limits

- Metadata, dependency/duplicate/cycle validation, side filtering, deterministic
  ordering, lifecycle cleanup and opt-in typed events.
- Namespaced items, blocks, creative categories, embedded original resources,
  shapeless recipes, commands and versioned block entities.
- Read-only compiled-mod entries in the existing Mods screen; authoritative
  server snapshots/actions and static-content compatibility checks before play.
- Generated zero/one/multiple-mod packs, source hash/toolchain locks and
  deterministic source ZIP packaging. Persistent IDs are namespaced strings.

Native code is **trusted executable code**, not sandboxed. Review crates and
their dependencies before compiling or running them. Disabling a linked content
mod generally requires rebuilding; removing content can make a world incompatible.
Client/server static content must agree. Test save compatibility before upgrading.

Mutable mod settings, creative pagination, dynamic/special item models, registry
remapping, arbitrary remote mod installation and a Create port are not provided.
Reproducibility covers source/dependency selection and source archive bytes;
bit-identical executables across hosts/linkers/paths are not promised.

## Build from source

The supported recipe targets **x86_64 Linux** with **Rust/Cargo 1.99.1** and
**Python 3.11+**, Git and a native C/C++ linker toolchain. `rust-toolchain.toml`
selects the same Rust version as `native-modpack.toml`. Other hosts need an
intentional source-lock update and separate validation.

On Ubuntu 24.04, engine CI also installs:

```sh
sudo apt-get install build-essential pkg-config libasound2-dev libudev-dev libvulkan1 mesa-vulkan-drivers
git clone --recurse-submodules https://github.com/natimys/horizon-native-mod-loader.git
cd horizon-native-mod-loader
python3 tools/check_engine_pin.py
```

For an existing clone, run `git submodule update --init --recursive`. Do not use
`git submodule update --remote`: builds use the reviewed commit, not a moving
branch. A plain GitHub source ZIP does not contain the engine submodule.

The API/runtime can be tested without Minecraft files or an initialized engine:

```sh
cargo fetch --manifest-path native-mods/Cargo.toml --locked
cargo test --manifest-path native-mods/Cargo.toml --workspace --all-features --locked --offline
```

To build the integrated engine, first fetch its locked dependencies, verify the
source recipe, then choose a build:

```sh
cargo fetch --manifest-path horizon/Cargo.toml --locked
python3 tools/native_pack_release.py verify
python3 tools/native_pack_release.py build --variant native
# Optional alternative: include Workshop's gameplay example.
python3 tools/native_pack_release.py build --variant native --features gameplay-example
# Feature-off regression build:
python3 tools/native_pack_release.py build --variant vanilla
```

The wrapper builds `horizon-client` and `horizon-ctl` under `horizon/target/debug/`
with no default audio/controller features, using `--locked --offline`. Add
`--release` for release-profile output. Each variant replaces the same output
paths. The default native pack includes Hello; `test-pack` adds Companion and
`gameplay-example` adds Workshop. Use these example features separately in game
runs because the current Creative screen has a fixed category capacity.

## Install and run a custom build

There is no binary installer in this first source publication. Keep the compiled
binary with your reviewed pack and use a separate data directory. Minecraft
resources are **not included** and are not required for the asset-free unit/CI
checks. To play, prepare resources separately from your own licensed installation
following [Horizon's build instructions](horizon/README.md#build-from-source).
The tools here do not download or redistribute Minecraft files.

For an isolated smoke run, set `RESOURCES` to your prepared resource directory,
then run from the outer repository:

```sh
project_dir=$PWD
scratch=$(mktemp -d)
export HORIZON_HOME="$scratch/game" HORIZON_LAUNCHER_HOME="$scratch/launcher"
export XDG_DATA_HOME="$scratch/data" XDG_CONFIG_HOME="$scratch/config"
export HORIZON_LAUNCHER_NO_SELF_UPDATE=1 HORIZON_NO_AUDIO=1
export HORIZON_NO_DISCORD=1 HORIZON_NO_TELEMETRY=1
mkdir -p "$HORIZON_HOME" "$HORIZON_LAUNCHER_HOME" "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"
cd "$scratch"
"$project_dir/horizon/target/debug/horizon-client" --resources "$RESOURCES"
```

Use the engine's documented `--dedicated` mode for a server, with separate homes
and explicit EULA acceptance by the operator. Do not point test runs at personal
worlds/settings. Mods-screen entries describe permanently linked code; they are
not runtime enable/disable switches.

## Create your first mod

1. Copy `docs/native-modding/templates/basic/` to `native-mods/mods/example/`.
   The template exports `create() -> Box<dyn NativeMod>`, metadata and an
   initialization hook that queues a log message. Set your package, mod ID,
   authors and license. Its only dependency is `horizon-native-api`.
2. Add `"mods/example"` to the native workspace `members` and add this entry to
   `native-modpack.toml`:

   ```toml
   [[mods]]
   package = "example-mod"
   path = "native-mods/mods/example"
   version = "0.1.0"
   ```

3. Run `python3 tools/native_modpack.py generate`, then
   `cargo test --manifest-path native-mods/Cargo.toml --workspace --all-features --offline`
   and `python3 tools/check_native_boundaries.py`. The initial test updates the
   native Cargo lock for your newly added crate.
4. Run `cargo check --manifest-path horizon/Cargo.toml -p horizon-client --bin horizon-client --no-default-features --features native-mods --offline`
   to update the engine lock for the new composition. Review and commit any
   engine `Cargo.lock` change separately. Stage your reviewed new mod source and
   generated files explicitly, then run `python3 tools/native_pack_release.py lock`.
   Review the new source lock; run tests, commit, and rebuild the native variant.

An idle mod need not subscribe to ticks. For content and gameplay, read Hello
and Workshop and the [public API policy](docs/native-modding/API_POLICY.md).
Use original synthetic fixtures; do not copy player resources into your mod.
The complete recipe/update/package workflow is in
[TOOLCHAIN.md](docs/native-modding/TOOLCHAIN.md).

## Validation, contribution and license

[CI](.github/workflows/native.yml) tests the native workspace and the pinned
engine in vanilla, native and combined WASM/native configurations without
proprietary game resources. Historical acceptance evidence is in
[PHASE4_VALIDATION.md](docs/native-modding/PHASE4_VALIDATION.md); it is distinct
from a successful hosted run of this repository. See also the
[architecture](docs/native-modding/ARCHITECTURE.md),
[engine-change ledger](docs/native-modding/HORIZON_INTEGRATION_MAP.md) and
[publication audit](docs/PUBLICATION_AUDIT.md).

Contributions follow [CONTRIBUTING.md](CONTRIBUTING.md),
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) and [SECURITY.md](SECURITY.md).
Original outer-project code and example artwork are [MIT licensed](LICENSE).
Horizon retains its own MIT attribution and third-party component licenses;
see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). This project grants no rights
to Minecraft game assets or trademarks.
