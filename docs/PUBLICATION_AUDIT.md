# First-publication audit — 2026-10-30

Scope: local preparation of an independent source project and its engine fork.
No GitHub repository was created, no content uploaded and no push performed.
Repository names were confirmed by the owner:
`natimys/horizon-native-mod-loader` and `natimys/horizon-native-engine`.
See [PUBLISHING.md](PUBLISHING.md) for the dependency-first publication sequence.

## Baseline and preservation

| Repository | Initial state |
| --- | --- |
| Outer | `master` at `dac1fcf`; 34 commits, no remotes, no gitlink; `horizon/` ignored |
| Engine | `main` at `f84396c94ebc52f9abb072522b8779a8834362b7`; 22 commits ahead of upstream `cad2a29d2ad60a9fa001f8e1a087becf13665519` |

The original outer `.agents/`, `.codex/`, `README-SETUP.md` and engine
`horizon_official_server.txt` were untracked. They remain local, unmodified and
ignored; they were not staged as project contributions. Private `game-resources/`
was neither opened for this audit nor copied. No player home or save was used.
No existing commit was amended, squashed, rebased or removed. Commit authorship
and public author email metadata remain part of the preserved Git history.

## Source and historical object audit

`tools/audit_publication.py` reads reachable Git blobs and commit messages,
including deleted files. It reports locations/object IDs, never credential
values. The native history contains only source/docs/tooling and two tiny original
PNG examples: `marker.png` (109 bytes), `token.png` (145 bytes). The original-art
provenance is documented in the phase 2 integration/validation notes.

No proprietary game resources, jars, recovered sources, personal saves,
credentials, local configuration or build output were found in the outer history.
The project now ignores local agent/editor config, credentials, private resources,
archives, binaries, player data and generated outputs. The audit checks forbidden
historical paths, high-signal credential patterns, personal absolute paths and
objects over 5 MiB. It is a heuristic audit, not a proof of secret absence or a
copyright-origin detector; no dedicated entropy-based secret scanner was installed.

The engine's own `tools/check_public_tree.py --history` passed across **all local
refs**, including upstream tags/remote branches. The object inventory covered
**9,464 commits / 32,499 blobs / 5,432 historical paths** before preparation.
It identified **789 historical binary blobs** across all refs, chiefly upstream
images/fonts. Only **253** belong to the integration HEAD ancestry. The largest
in that ancestry is `website/docs/widths/1920.png`, **1,801,222 bytes**; there are
no objects above the 5 MiB threshold in that ancestry. An upstream side branch
contains `website/full-390-before-after.png`, **6,349,398 bytes**; that branch is
outside the selected publication history and must not be mirror-pushed.

The broad engine scan reported **206** path-like matches and **7** credential-URL
matches (counts are distinct historical objects, before the prose-boundary fix).
All findings reachable from integration HEAD were already reachable from the
pinned upstream base; **no integration-introduced finding** was present. The URL
matches are proxy/redaction/host-parsing unit-test fixtures. Path matches include
upstream benchmark reports, documentation, redaction fixtures and old commit
messages. These inherited public paths remain in upstream ancestry; preserving
history means this publication cannot promise a path-free engine history.
There were no matches for the audited GitHub token, AWS access-key ID or private
key patterns. Raw match values and local filesystem inventories are not published.
The follow-up full-ref scan at fork preparation commit `f3cc87030`, with additional
service-token and literal-secret-assignment patterns, covered 9,465 commits and
32,502 blobs. It reported 198 path matches after the prose-boundary fix, the same
7 test-URL objects and the same unrelated-branch large image; no additional token
or literal-secret matches. The subsequent two workflow fixture-path edits were
checked separately and introduce no credential/path findings.

## Licenses and provenance

The outer project now uses MIT with contributor attribution; all native crate
manifests, generated pack metadata and the author template declare MIT.
The native workspace has no external Cargo dependencies beyond Rust std.
Source ZIPs include the license and third-party notices, with regression coverage.
MIT requires retention of its notice ([license text and conditions](https://choosealicense.com/licenses/mit/)).

The engine retains upstream's MIT notice for Ludovic Marie and the Horizon
contributors, original history, `AGENTS.md`, `CONTRIBUTING.md` and third-party
licenses/notices. The fork guide credits downstream contributions separately.
wgpu-hal, fonts and icons retain their existing component notices. The outer MIT
license does not relicense all transitive engine dependencies. This is a source
publication; binary distribution needs a separate exact-target license inventory.
No rights to Minecraft assets or trademarks are asserted.

## Integration and CI preparation

Engine preparation commits: `f3cc87030` and `d303c4d00` on
`codex/native-integration`; the final submodule pin is
`d303c4d009811d83071bbaf26411ff5fbd4cd454` (matches the gitlink/source lock).
Changes are README/fork documentation, a local-note ignore rule, and relocation of 14
upstream workflow files into `.github/upstream-workflows/`. Their bytes are
unchanged. This prevents inherited release/deploy/private-runner jobs from
executing in the downstream fork. Two Python test modules now read those preserved
fixtures at their new location, retaining all assertions. No engine Rust code, public Mod API or gameplay
hook was added or changed in this preparation step.

The outer project tracks a valid mode-160000 gitlink plus HTTPS `.gitmodules`
URL, agrees with the source lock and checks the actual engine checkout. Its CI
recursively clones that exact revision; the old unavailable-upstream/variable
fallback is removed. The `native` job audits outer ancestry; vanilla/native/
WASM-native jobs use locked offline Cargo commands after explicit fetching.
Test homes stay isolated; no proprietary resources or deployment secrets are
needed. The engine fork must be published first; public URL availability and
hosted Actions cannot be verified before publication is authorized.

## Validation record

| Check | Actual result |
| --- | --- |
| Outer `python3 -m unittest discover -s tests` | PASS, 22 tests |
| Native workspace all-feature locked/offline tests | PASS, 29 tests |
| Native rustfmt and all-target/all-feature Clippy `-D warnings` | PASS |
| Generated zero/one/two/Hello+Workshop packs | PASS, four scratch compositions |
| Independent author template | PASS, API-only tests and Clippy |
| Source recipe verify and feature-off dependency boundary | PASS |
| Source ZIP roundtrip, repeated byte equality, checksums, license/notices inclusion | PASS; exported sources rebuild/test with a separate exact engine checkout |
| New independent recursive local clone | PASS; gitlink/lock/checkout, feature-off graph, outer audit and both `git fsck --full` checks; private/local files absent |
| Horizon vanilla client lib/game/ctl | PASS: 669 + 153 + 18; 21 lib + 3 game tests ignored |
| Horizon native + Workshop lib/game/ctl | PASS: 677 + 160 + 18; same ignores |
| Combined WASM/native + Workshop lib/game/ctl | PASS: 677 + 163 + 18; same ignores |
| Sequential renderer suite | PASS: 526, 15 ignored |
| Original WASM `runtime,pack` | PASS: 22 unit + 4 API reference + 32 real sandbox; 1 performance test ignored |
| Archived-workflow contract modules | PASS: 45 tests, no assertion removed |
| Full upstream Python tooling suite after fixture-path correction | PASS: 416 tests |
| Upstream public-tree/history guard | PASS across all local refs |
| GitHub CI/community YAML and local README/community links | PASS |

The first socket-dependent client/renderer runs failed with sandbox `EPERM`
(6 client library tests per variant, 2 renderer HTTP fixtures). The same suites
passed with local sockets permitted and isolated temporary homes. No player
resources were used. The initial upstream Python tooling run exposed 7 missing
workflow fixture paths after archiving; the two path updates fixed those errors.

The installed `stable` alias reports the exact rustc/Cargo 1.99.1 identities in
the source lock. Local checks used `RUSTUP_TOOLCHAIN=stable` after the separately
named `1.99.1` toolchain attempted an unavailable local installation. CI explicitly
installs the named version. No toolchain download success is claimed locally.

**Not run / pending:** public anonymous GitHub recursive clone and hosted Actions
(repositories/push not authorized); actionlint (not installed); fresh asset-based
game scenarios and Java differential parity (no gameplay/Rust changes in this
step, no proprietary resources used). Prior phase acceptance remains historical
evidence in `native-modding/PHASE4_VALIDATION.md`.
