# Horizon Native Modding — workspace instructions

This is the OUTER integration workspace, not upstream Horizon. Run Codex from this directory.

## Directory ownership

- `horizon/`: independently versioned upstream Horizon Rust game checkout. Preserve its `AGENTS.md`, `CONTRIBUTING.md`, build instructions, copyright safeguards, and vanilla parity harnesses. Do NOT replace its existing WASM loader.
- `game-resources/`: privately prepared/player-owned Minecraft resources. Treat as read-only and confidential; never add to Git, copy into examples, upload, post in PRs, or use for unit-test fixtures.
- `docs/`: integration notes, API decisions, implementation plan, change logs.
- `native-mods/`: independent, std-only Cargo proof-of-concept for API/runtime/static pack, NOT yet integrated into Horizon.
- `.agents/skills/`: repository-scoped workflows for Codex.
- `.codex/`: project-local Codex config and specialized agents.

## Purpose

Build an opt-in, compile-time linked Rust modding path alongside Horizon's existing systems. Mod authors write Rust crates; Cargo compiles and statically links them into a custom Horizon game/server binary. No WASM for this NEW path, no Java jars, no dlopen/dylib plugins, and no runtime source patching. Do not represent this as support for loading arbitrary mods into a prebuilt executable.

## NON-NEGOTIABLE ARCHITECTURAL BOUNDARY

1. Engine changes are allowed ONLY to expose the smallest generic hook, registry adapter, or public API boundary needed by several independent mods. Prefer adapters and new crates to invasive engine edits.
2. The engine MUST NOT know concrete mods, import their types, test their identities, store mod-specific fields in `Game`, or implement mod behavior directly. Only generic mod runtime/bridge code may call generic hooks.
3. A mod MUST NOT depend on `horizon-client`, `horizon-world`, a private `Game` type, engine implementation modules, or internal renderer structures. A mod may depend only on a stable `horizon-native-api`, other explicitly declared mod libraries, and Rust libraries.
4. NEVER use mixin/AST rewriting, monkey patching, unsafe global pointers, raw `Game` references, or unrelated direct engine code changes to implement a mod feature. If the API lacks a capability, propose a small general-purpose addition, document it, and test it.
5. Design dependency direction to avoid cycles: `mod crates -> native-api`; `native-runtime -> native-api`; `native-pack -> native-runtime + mod crates`; `Horizon integration -> native-pack / runtime bridge`. The `native-api` must not depend on engine internals.
6. The default, no-native-mod build must preserve vanilla behavior and existing parity tests. The original WASM loader remains independent; a shared presentation UI may later show both kinds of mods.
7. Native mods are trusted executable code with full process privileges, not sandboxed. Never describe them as safe to install from unknown sources. Disabling a statically linked content mod generally requires rebuilding.

## Delivery protocol

Before editing engine code:
- Read `horizon/AGENTS.md`, `horizon/CONTRIBUTING.md`, `horizon/docs/architecture.md`, `horizon/docs/mods.md` if present, then discover the actual workspace layout (upstream may have changed).
- Create/update `docs/native-modding/HORIZON_INTEGRATION_MAP.md` with real paths, functions, callsites, existing registry and menu abstractions; distinguish observations from proposed changes.
- Read `docs/native-modding/ARCHITECTURE.md` and `ROADMAP.md`.
- Write a short change plan showing each proposed engine insertion, why necessary, and why a public API/adapter cannot avoid it.
- Keep changes in independently testable phases. First implement and test metadata/lifecycle and a hello-world mod; registries and UI follow; only then plan Create.

For EVERY engine edit:
- Add or extend a generic contract, not a concrete mod special case.
- Keep a written engine-change ledger in `HORIZON_INTEGRATION_MAP.md` (`file`, `entry point`, `reason`, `alternative considered`, `test`).
- Prefer deterministic compile-time registration and fail-fast duplicate ID/dependency checks.
- Keep content IDs namespaced and stable across saves; do not serialize transient integer IDs as persistent identifiers.
- Keep registration distinct from world-running lifecycle; freeze content registries before worlds are created or loaded.
- Separate common/server/client hooks and do not invoke client UI code on a dedicated server.
- Ensure event ordering, ownership and error behavior are documented and tested.

## Safety, builds, and review

- Never commit Mojang assets, decompiled code, jars, `game-resources/`, prepared resources, or personal saves. Respect upstream's own file-guard scripts.
- All game/launcher test runs MUST use isolated temporary `HORIZON_HOME`, `HORIZON_LAUNCHER_HOME`, `XDG_DATA_HOME`, `XDG_CONFIG_HOME`; set `HORIZON_LAUNCHER_NO_SELF_UPDATE=1`, `HORIZON_NO_AUDIO=1`, and upstream's additional launcher scratch environment variables when applicable. Never alter real player saves/settings.
- Check `git status --short` in `horizon/` before editing; do not discard or overwrite uncommitted user changes; do not run destructive Git commands.
- Run the narrowest relevant `cargo fmt`, `cargo check`, tests, and vanilla/parity harnesses appropriate to the files changed. Never claim a test passed unless actually run. If toolchains or proprietary assets are unavailable, report that explicitly.
- Do not bulk-format upstream code or modify unrelated modules.
- Report at task end: public API added, exact engine changes and justification, test results, limitations, next smallest step.

## Mandatory Git commit policy

This policy applies to all further Horizon Native Mod Loader development and all
future agent sessions in this workspace, including the independent `horizon/`
repository. One completed engineering step means one commit.

1. At the start of work, check `git status --short` in every repository you will
   touch. Identify existing uncommitted changes; never claim them as your own or
   mix them into your commits.
2. Commit after each small, logically complete change. Do not accumulate dozens
   of changes or combine independent work to save time.
3. Keep every commit atomic, with a clear purpose and the minimum relevant files.
   Commit new APIs, bug fixes, tests, refactors, documentation and engine
   integrations separately when they are independently complete changes. Include
   tests needed to validate an implementation in that implementation's commit.
4. Run relevant tests and checks before committing whenever possible. Never
   knowingly commit a broken state without an explicit reason and a clear note
   in the commit message; report checks that could not run.
5. Use Conventional Commits, for example:
   - `feat(native-api): add item registry`
   - `feat(native-ui): implement mod list adapter`
   - `fix(native-runtime): resolve lifecycle cleanup`
   - `test(native-api): cover duplicate registration`
   - `refactor(native): simplify registry snapshot`
   - `docs(native): update phase 2 validation`
6. Stage explicit paths belonging to the current commit. Avoid `git add .` unless
   necessary; inspect the staged diff before committing.
7. Never commit others' uncommitted changes, secrets, temporary files, build
   outputs, proprietary resources or personal saves.
8. Preserve existing Git history. Do not amend, rebase or otherwise rewrite it
   without separate user authorization.
9. Never run `git push` without an explicit user instruction.
10. At the end of every stage, report the created commit range (and repository)
    and actual test/check results, including failures and checks not run.

## Skills

Use `$horizon-audit` before touching upstream integration, `$native-mod-api` for lifecycle/hooks/crate boundaries, `$registry-and-ui` for content tabs/mod list, and `$mod-validation` for acceptance checks. Skills never override the non-negotiable boundary rules above.
