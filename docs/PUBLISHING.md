# First GitHub publication

Preparation is local only. Creating repositories, changing GitHub settings and
pushing require the owner's separate authorization. The commands below are a
future operator checklist, not a publication script.

## Repositories

1. Create **natimys/horizon-native-engine** as a public GitHub fork of
   **horizon-rust/horizon**, retaining its upstream history. Keep Actions disabled
   during setup. The integration branch is `codex/native-integration`; upstream
   `main` is preserved separately. The new tip archives inherited upstream jobs
   and documents the fork. Do not import or mirror all local upstream refs/tags.
2. Create **natimys/horizon-native-mod-loader** as an empty public repository;
   do not initialize a README/license or create unrelated history. Publish the
   outer history as `main`. Use a description that says independent/experimental,
   and topics such as `rust`, `horizon`, `modding`, `native-mods`.

The outer remote `origin` targets the loader. In the submodule, `origin` targets
the engine fork and `upstream` targets horizon-rust/horizon. The original local
engine `main` branch and all existing tags/remote-tracking refs were retained.
Only the integration branch is intended for a downstream push.

## Publish the dependency first — only after authorization

Before pushing, inspect both statuses/staged diffs and rerun the public audits.
Inspect `docs/PUBLICATION_AUDIT.md`, including inherited upstream findings.
The intended future commands are:

```sh
# FIRST: integration history, no unrelated refs or tags.
git -C horizon -c push.followTags=false push origin HEAD:refs/heads/codex/native-integration
git ls-remote https://github.com/natimys/horizon-native-engine.git refs/heads/codex/native-integration
```

Confirm the advertised commit equals `native-modpack.lock`'s `horizon` value.
Set the fork default branch to `codex/native-integration`. Do not enable the
archived upstream release/deploy workflows. Review the fork branch contents and
public anonymous access before publishing the outer project:

```sh
# SECOND: existing outer history, without squash or force.
git -c push.followTags=false push origin HEAD:refs/heads/main
```

Set the outer default branch to `main`. A missing fork commit is a publication
blocker: do not replace its pin with upstream main or publish a dangling pointer.
Until the dependency is publicly reachable, the local gitlink is verifiable but
anonymous recursive cloning and hosted CI cannot be certified.

## Verify the public clone and settings

From an empty temporary directory, with credential helpers disabled, clone the
outer public URL using `--recurse-submodules`. Run `tools/check_engine_pin.py`,
`tools/native_pack_release.py verify` and the documented checks. Confirm the
engine checkout equals the gitlink/source lock without a local URL override.
Keep the temporary clone separate from player data.

Enable Actions in the outer repository with read-only default workflow token
permissions and no deployment secrets/private runners. Let the existing `native`
job and all three `horizon` matrix variants pass on GitHub before claiming a
hosted green build. CI does not need Minecraft resources. It pins checkout by
action SHA, Rust version, Cargo locks and exact engine gitlink; unavailable
dependencies fail instead of falling back. GitHub runner/apt environments remain
mutable, so binary bit-for-bit reproducibility is outside the contract.

Enable private vulnerability reporting, then require the native and engine
matrix checks for the outer default branch and prevent force pushes/deletion.
Use merge commits or fast-forwards that preserve atomic history; disable squash
and rebase merging for this project. No release assets or binaries are prepared
by this task. A binary release requires its own dependency/license review.

## Future engine updates

Fetch from `upstream`, review and merge explicitly, run relevant parity and
vanilla/native/WASM regressions, and commit the engine change atomically. Publish
the engine commit first. Update the outer gitlink and source lock together and
verify a fresh public recursive clone before publishing the outer update. Keep
the engine-change ledger current. Never use `git push --mirror`, `--all`,
`--force` or an unreviewed tag push for this setup.
