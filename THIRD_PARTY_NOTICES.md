# Licensing and source provenance

The outer project's original Rust code, Python tools, documentation, templates
and example artwork are offered under [MIT](LICENSE). Copyright belongs to the
named authors and contributors; Git history retains authorship. The two bundled
example textures (`token.png`, `marker.png`) are original synthetic example art,
as recorded in the integration and validation notes. Minecraft names and template
identifiers in descriptors refer to resources prepared separately by the player;
they do not include Minecraft resource bytes.

## Horizon engine

The `horizon/` submodule is an independently versioned derivative of
[horizon-rust/horizon](https://github.com/horizon-rust/horizon). Its MIT notice is
copyright (c) 2026 Ludovic Marie and the Horizon contributors. Keep its
[LICENSE](horizon/LICENSE), [third-party notices](horizon/THIRD_PARTY_NOTICES.md)
and component license files intact. Our integration patches retain upstream
history and attribution. The outer license does not replace those notices.

The fork vendors wgpu-hal (MIT OR Apache-2.0), Pixelify Sans (SIL OFL 1.1) and
Pixelarticons (MIT); the upstream notice identifies their paths and authors.
These components belong to the engine checkout, not the independent API/runtime.

## Rust dependencies and distributions

The checked-in native workspace currently uses only the Rust standard library
and this project's path dependencies. It has no third-party Cargo dependencies.
The engine has its own locked dependency graph and transitive licenses. A source
checkout downloads these through Cargo; the root MIT license does not relicense
them. Before distributing an engine binary, review the exact target/features'
Cargo graph and ship the corresponding third-party notices/licenses. This
publication prepares source repositories; it does not include game binaries or
claim that all engine dependencies are MIT.

## Excluded material

Minecraft game jars, decompiled sources, textures/audio, prepared resources,
personal worlds and local credentials are excluded. Obtain game resources from
your own licensed installation using Horizon's documented preparation process.
No Mojang or Microsoft license or trademark rights are granted by this project.
This is an independent project, not an official Horizon, Mojang or Microsoft
release and not endorsed by those projects or companies.
