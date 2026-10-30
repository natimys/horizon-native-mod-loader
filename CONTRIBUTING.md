# Contributing

Start with the README, AGENTS.md and `docs/native-modding/ARCHITECTURE.md`.
Discuss a focused change in an issue or pull request in this project's tracker.
Native integration bugs belong here, not in upstream Horizon's bug queue.

## Development boundaries

- Mods depend on the public API, never engine/runtime internals. Keep the API
  engine-independent and dependency direction acyclic. Concrete mod behavior
  belongs in mod crates, not generic engine hooks.
- Register content before world creation; use stable namespaced save IDs.
  Document ordering, side/ownership rules, error behavior and cleanup.
- Preserve the feature-off path and existing WASM loader. Before engine edits,
  follow the fork's original AGENTS.md/CONTRIBUTING.md and update the outer
  integration ledger with rationale, alternatives and relevant tests.
- Submit original work you have the right to license. Keep attribution and
  component license notices. Contributions use this project's MIT license unless
  explicitly discussed otherwise. No CLA or copyright assignment is required.
- Never commit Minecraft jars, recovered code, player resources, worlds,
  credentials, build output or machine-specific configuration. Use original
  synthetic test data. Run games only in fresh temporary game/launcher/XDG homes
  using the isolation environment required by AGENTS.md.

## Checks

Use the pinned toolchain and initialized submodule. After fetching locked Cargo
dependencies, run:

```sh
python3 tools/check_engine_pin.py
python3 tools/native_modpack.py check
python3 tools/native_pack_release.py verify
python3 tools/check_native_boundaries.py --feature-off
python3 -m unittest discover -s tests -v
cargo fmt --manifest-path native-mods/Cargo.toml --all -- --check
cargo test --manifest-path native-mods/Cargo.toml --workspace --all-features --locked --offline
cargo clippy --manifest-path native-mods/Cargo.toml --workspace --all-targets --all-features --locked --offline -- -D warnings
python3 tools/audit_publication.py --all-refs
```

Run the relevant engine/parity checks for engine changes; list checks that need
unavailable assets or toolchains instead of claiming success. The CI matrix is
the asset-free regression reference. For locked-source changes, explicitly
review and update the recipe lock; never regenerate it just to silence a failure.
New native source must be explicitly staged before locking.

## Commits and pull requests

Keep atomic Conventional Commits; include tests required by each implementation.
Preserve existing history. Do not squash, force-push, rebase published work or
rewrite history without separate authorization. Stage explicit paths and inspect
the staged diff. Never include someone else's uncommitted files.

Describe the concrete behavior, API/engine changes, compatibility implications
and actual validation. Disclose agent-assisted work and follow upstream's agent
attribution rules for engine contributions. A fork update must publish an
accessible engine commit before updating the outer gitlink. Do not reference an
unavailable private engine commit in a public PR. Review submodule changes as
code, not just as a hash bump.
