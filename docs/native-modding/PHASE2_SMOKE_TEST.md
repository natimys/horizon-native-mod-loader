# Phase 2 manual smoke in a graphical session

Run from an ordinary terminal inside the logged-in graphical session. Preserve
DISPLAY, WAYLAND_DISPLAY and **XDG_RUNTIME_DIR**: changing the latter hides the
compositor socket. The Codex sandbox can see that socket but its connect syscall
returns EPERM. The compositor itself works; isolated launches outside the sandbox
passed actual Vulkan rendering. A missing-compositor error in that sandbox is an
environment failure, not evidence of a native-loader bug.

Only original native example resources are bundled. Supply your own read-only
prepared resource root; do not copy it into examples/tests or add it to Git.

```sh
workspace="$PWD"
export HORIZON_TEST_RESOURCES=/absolute/path/to/your/prepared/resources
cargo build --manifest-path "$workspace/horizon/Cargo.toml" \
  -p horizon-client --bin horizon-client --bin horizon-ctl \
  --no-default-features --features native-mods,horizon-native-pack/test-pack --offline

scratch=$(mktemp -d /tmp/horizon-native-manual.XXXXXX)
export HORIZON_HOME="$scratch/game"
export HORIZON_LAUNCHER_HOME="$scratch/launcher"
export XDG_DATA_HOME="$scratch/data"
export XDG_CONFIG_HOME="$scratch/config"
export HORIZON_LAUNCHER_TRASH="$scratch/trash"
export HORIZON_LAUNCHER_CLIPBOARD="$scratch/clipboard"
export HORIZON_LAUNCHER_DESKTOP="$scratch/desktop"
export HORIZON_LAUNCHER_NO_SELF_UPDATE=1 HORIZON_NO_AUDIO=1
export HORIZON_NO_DISCORD=1 HORIZON_NO_TELEMETRY=1
mkdir -p "$HORIZON_HOME"
cd "$HORIZON_HOME"
"$workspace/horizon/target/debug/horizon-client" \
  --resources "$HORIZON_TEST_RESOURCES" --world NativeManual --seed 123 \
  --creative --test-terrain --width 960 --height 640 --render-distance 2 \
  --control-socket "$scratch/ctl.sock"
```

Use `--features native-mods` for the default single-HelloWorld pack. The optional
second mod is selected in the external pack, without editing engine source.
The default server-thread path must be tested; `--in-frame` is an additional
comparison, not a substitute. `--offscreen` renders real frames without a visible
window and was used for the automated smoke recorded in VALIDATION.md.

1. Open Options > Mods. Both Native Companion and Hello from native Rust should
   appear as statically linked Native Rust. Open each details page; check ID,
   version, dependencies, side and rebuild/trust notice, scrolling and Back/Done.
   There must be no runtime enable switch for either linked content mod.
2. Enter the world and open Creative. Companion's blue C icon and Hello's orange
   H token must occupy distinct free tabs. Existing vanilla tabs keep positions.
   Each native tab has its own item; take one of each into different hotbar slots.
3. Confirm `hello_mod:token` and `companion:marker` using the control inventory
   command below. Search by item name and namespaced ID in Creative Search.
4. Switch language between English and Russian. Check titles, item tooltips,
   icons, hotbar, held-item rendering and existing vanilla content.
5. Press F3+T. Wait for reload; check both tabs/icons again. Registries do not
   duplicate and mod initialization is not replayed by a resource reload.
6. Save and Quit normally. Relaunch with the **same** scratch exports and world.
   Both item IDs, counts and slots must survive. All worlds/settings remain in
   this scratch directory. Never point this test at personal saves.
7. Optional composition migration: create/save Hello Token with the default pack,
   quit, rebuild with test-pack, and reopen the same scratch world. Companion sorts
   before HelloWorld and shifts protocol numbers; the saved Hello Token must keep
   its identity. Reverse constructor order is also covered by automated tests.

In another terminal, use the socket of this test only:

```sh
"$workspace/horizon/target/debug/horizon-ctl" --socket "$scratch/ctl.sock" inventory
"$workspace/horizon/target/debug/horizon-ctl" --socket "$scratch/ctl.sock" save
"$workspace/horizon/target/debug/horizon-ctl" --socket "$scratch/ctl.sock" quit
```

For TCP compatibility, open this scratch world to LAN on an ephemeral port. A
client with the same test-pack should join. In a **different** isolated scratch
home, a client built without native content or with only HelloWorld must be refused
before play with a static-content mismatch; it must not receive a wrong item.
The actual server rejection of absent and different identities was tested using a
bounded local protocol probe. No remote servers, player accounts or personal
worlds were used. Close the temporary LAN listener afterwards.

Unperformed supplemental checks in this iteration: normal visible-window mouse/
keyboard session, in-frame gameplay comparison, full dedicated-world run, and
full Java differential parity. The mandatory phase 2 rendered checks did run with
real offscreen Vulkan frames; these remaining checks are not described as passed.

Known scope: plain static model items; vanilla component templates; common mods
for persistent content; existing 2x7 creative capacity (overflow fails atomically).
No native settings API, gameplay actions, arbitrary item model trees or registry
remapping are implied. Native code is trusted process code and requires rebuilding.
