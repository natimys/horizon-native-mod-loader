# Security policy

This project is experimental (0.1.0 / API epoch 1). Only the latest reviewed
source revision is maintained; no older-version security support is promised.

Native mods and their Rust dependencies/build scripts execute trusted code with
the user's process privileges. The loader provides no sandbox or hostile-mod
containment. Hashes, source locks and dependency checks do not make unknown code
safe. Review code before building or running it. The separate Horizon WASM loader
has its own security contract.

For a vulnerability, use this repository's GitHub **Security → Report a
vulnerability** feature once private reporting has been enabled. If unavailable,
contact the maintainer privately through their public GitHub profile; you may
open a content-free issue asking for a private channel. Do not put credentials,
exploit payloads, personal worlds or proprietary game files in public issues.
There is no response-time or bounty promise. Upstream-only findings should use
Horizon's own reporting process; do not send private reports to public comments.

If a credential is committed, revoke/rotate it first and contact the maintainer.
Deleting the current file does not remove historical Git objects. Any history
remediation requires an explicit decision and coordination with existing clones.
