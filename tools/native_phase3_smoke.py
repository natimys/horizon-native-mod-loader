#!/usr/bin/env python3
"""Real isolated phase 3 acceptance; requires a gameplay-example client build.

No assets/fixtures are bundled. Supply a read-only player-owned resource root.
Keep DISPLAY/WAYLAND_DISPLAY/XDG_RUNTIME_DIR; run in a graphical session.
"""
import argparse
import json
import math
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time


def eventually(read, predicate, timeout=60):
    deadline = time.monotonic() + timeout
    last = None
    while time.monotonic() < deadline:
        last = read()
        if predicate(last):
            return last
        time.sleep(0.15)
    raise AssertionError(f"timed out: {last}")


class Session:
    def __init__(self, args, label):
        self.args = args
        self.root = Path(tempfile.mkdtemp(prefix=f"horizon-phase3-{label}-"))
        self.home = self.root / "game"
        self.home.mkdir()
        self.env = os.environ.copy()
        for key, part in {
            "HORIZON_HOME": "game", "HORIZON_LAUNCHER_HOME": "launcher",
            "XDG_DATA_HOME": "data", "XDG_CONFIG_HOME": "config",
            "HORIZON_LAUNCHER_TRASH": "trash",
            "HORIZON_LAUNCHER_CLIPBOARD": "clipboard",
            "HORIZON_LAUNCHER_DESKTOP": "desktop",
        }.items():
            self.env[key] = str(self.root / part)
        for key in ["HORIZON_LAUNCHER_NO_SELF_UPDATE", "HORIZON_NO_AUDIO",
                    "HORIZON_NO_DISCORD", "HORIZON_NO_TELEMETRY"]:
            self.env[key] = "1"
        self.env["RUST_LOG"] = "info"
        self.socket = self.root / "ctl.sock"
        self.process = None
        self.iteration = 0

    def call(self, *words):
        result = subprocess.run(
            [str(self.args.ctl), "--socket", str(self.socket), *map(str, words)],
            env=self.env, cwd=self.home, capture_output=True, text=True,
            timeout=65, check=False,
        )
        if result.returncode:
            raise AssertionError(f"{words}: {result.stdout} {result.stderr}")
        value = json.loads(result.stdout)
        if "error" in value:
            raise AssertionError(value)
        return value

    def start(self, extra):
        self.iteration += 1
        if self.socket.exists():
            self.socket.unlink()
        self.process = subprocess.Popen(
            [str(self.args.binary), "--resources", str(self.args.resources),
             "--render-distance", "2", *extra], cwd=self.home, env=self.env,
            stdout=(self.root / f"run-{self.iteration}.log").open("w"),
            stderr=subprocess.STDOUT,
        )

    def client(self, extra):
        self.start(["--offscreen", "--no-pause", "--width", "640", "--height", "480",
                    "--control-socket", str(self.socket), *extra])
        def alive():
            assert self.process.poll() is None, f"game exited: {self.root}"
            return self.socket.exists()
        eventually(alive, bool, 180)
        self.call("wait-ready", 60)

    def stop(self, dedicated=False):
        if self.process is None or self.process.poll() is not None:
            return
        if dedicated:
            self.console("stop")
        else:
            self.call("quit")
        assert self.process.wait(timeout=90) == 0

    def console(self, text):
        with (self.root / "console.fifo").open("w") as stream:
            stream.write(text + "\n")


def counter(session, pos):
    return eventually(lambda: session.call("block", *pos),
                      lambda b: b.get("block_entity", {}).get("id") == "workshop:counter_data")


def place_command(session):
    p = session.call("state")["player"]["position"]
    pos = [math.floor(p[0]) + 2, math.floor(p[1]), math.floor(p[2])]
    session.call("chat", "/workshop:place")
    value = counter(session, pos)
    first = value["block_entity"]["fields"]["visits"]
    value = eventually(lambda: counter(session, pos),
                       lambda b: b["block_entity"]["fields"]["visits"] > first)
    return pos, value


def craft_and_place(session, pos):
    session.call("command", "clear")
    session.call("command", "gamemode survival")
    eventually(lambda: session.call("state"), lambda s: s["gamemode"] == "survival")
    session.call("give", "minecraft:stick", 1)
    session.call("give", "minecraft:cobblestone", 1)
    eventually(lambda: session.call("inventory"), lambda i: len(i["counts"]) == 2)
    session.call("open-inventory")
    for source, target in [(0, 100), (1, 101)]:
        session.call("click-slot", source)
        session.call("click-slot", target)
    eventually(lambda: session.call("ui"), lambda ui: any(
        w["id"] == "slot-200" and w.get("stack", {}).get("item") == "workshop:counter"
        for w in ui["widgets"] if w.get("stack") is not None))
    session.call("click-slot", 200)
    session.call("click-slot", 2)
    session.call("close")
    eventually(lambda: session.call("inventory"),
               lambda i: i["counts"].get("workshop:counter") == 1)
    session.call("command", "gamemode creative")
    eventually(lambda: session.call("state"), lambda s: s["gamemode"] == "creative")
    time.sleep(0.5)
    session.call("break", *pos, "--timeout", 20)
    session.call("place", *pos, "--item", "workshop:counter")
    value = counter(session, pos)
    eventually(lambda: session.call("chat-log"), lambda chat: all(
        any(text in line for line in chat["chat"])
        for text in ["Workshop broke workshop:counter", "Workshop placed workshop:counter"]))
    return value



def reload_and_catalog(session,pos):
    log=session.root / f"run-{session.iteration}.log"
    before_renderers=log.read_text().count("\nHorizon renderer:")
    before_init=log.read_text().count("HelloMod initialized")
    session.call("key","f3","down")
    session.call("key","t","tap")
    session.call("key","f3","up")
    # Reload may reuse the device/pipelines; a new renderer still logs its adapter.
    eventually(lambda:log.read_text(),lambda s:s.count("\nHorizon renderer:")>before_renderers,60)
    assert "resource reload failed" not in log.read_text()
    assert log.read_text().count("HelloMod initialized")==before_init
    counter(session,pos)
    session.call("options","mods")
    labels=[w.get("label","") for w in session.call("ui")["widgets"]]
    assert "Native Workshop 0.1.0" in labels and "Hello from native Rust 0.1.0" in labels,labels
    session.call("close")


def local(args):
    session = Session(args, args.mode)
    extra = ["--world", "Phase3", "--seed", "123", "--creative", "--test-terrain"]
    if args.mode == "in-frame":
        extra += ["--in-frame"]
    guest = None
    try:
        session.client(extra)
        # Vanilla commands make a clear reachable fixture in this scratch world;
        # random spawn terrain must not hide the counter behind another block.
        session.call("command", "fill -3 79 -3 5 84 3 minecraft:air")
        session.call("command", "fill -3 79 -3 5 79 3 minecraft:stone")
        session.call("command", "tp @s 0.5 80 0.5")
        eventually(lambda: session.call("state"),lambda s: abs(s["player"]["position"][1]-80)<0.01)
        pos, _ = place_command(session)
        craft_and_place(session, pos)
        before = eventually(lambda: counter(session,pos),lambda b:b["block_entity"]["fields"]["visits"]>=1)["block_entity"]["fields"]["visits"]
        session.call("save")
        session.stop()
        session.client(extra)
        restored = counter(session, pos)
        assert restored["block_entity"]["fields"]["visits"] >= before
        if args.reload:
            reload_and_catalog(session,pos)
        if args.mode == "integrated":
            port = session.call("lan", "open", 0, "commands")["port"]
            subprocess.run([sys.executable, str(Path(__file__).with_name("native_static_pack_probe.py")),
                            "--port", str(port)], check=True)
            guest = Session(args, "lan-guest")
            guest.client(["--server", f"127.0.0.1:{port}", "--name", "Phase3Guest"])
            initial = counter(guest, pos)["block_entity"]["fields"]["visits"]
            eventually(lambda: counter(guest, pos),
                       lambda b: b["block_entity"]["fields"]["visits"] > initial)
            guest_pos, _ = place_command(guest)
            counter(session, guest_pos)
            guest.stop()
            session.call("lan", "close")
        session.call("save")
        print(json.dumps({"mode": args.mode, "result": "PASS", "scratch": str(session.root),
                          "position": pos, "restored": restored}), flush=True)
    finally:
        if guest:
            guest.stop()
        session.stop()


def dedicated(args):
    if not args.eula_already_accepted:
        raise SystemExit("Dedicated run requires --eula-already-accepted after human confirmation")
    server, client = Session(args, "dedicated"), Session(args, "dedicated-client")
    try:
        (server.home / "eula.txt").write_text("eula=true\n")
        (server.home / "server.properties").write_text(
            "online-mode=false\nwhite-list=false\nserver-ip=127.0.0.1\nview-distance=2\nsimulation-distance=2\n"
            'gamemode=creative\nlevel-type=minecraft:flat\ngenerator-settings={"layers":'
            '[{"block":"minecraft:bedrock","height":1},{"block":"minecraft:dirt","height":2},'
            '{"block":"minecraft:grass_block","height":1}],"biome":"minecraft:plains"}\n')
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        fifo = server.root / "console.fifo"
        os.mkfifo(fifo)
        # Keep FIFO readable/writable so there is no premature console EOF.
        with os.fdopen(os.open(fifo, os.O_RDWR), "rb", buffering=0) as stream:
            server.iteration += 1
            server.process = subprocess.Popen(
                [str(args.binary), "--resources", str(args.resources), "--render-distance", "2",
                 "--dedicated", "--nogui", "--port", str(port), "--world", "Phase3"],
                cwd=server.home, env=server.env, stdin=stream,
                stdout=(server.root / "run-1.log").open("w"), stderr=subprocess.STDOUT)
        eventually(lambda: (server.root / "run-1.log").read_text(), lambda s: "Done (" in s, 180)
        client.client(["--server", f"127.0.0.1:{port}", "--name", "Phase3Dedicated"])
        pos, value = place_command(client)
        assert value["block_entity"]["version"] == 1
        client.stop()
        server.console("save-all")
        server.stop(dedicated=True)
        print(json.dumps({"mode": "dedicated", "result": "PASS", "scratch": str(server.root),
                          "position": pos, "replicated": value}), flush=True)
    finally:
        client.stop()
        server.stop(dedicated=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--mode", choices=["integrated", "in-frame", "dedicated"], required=True)
    parser.add_argument("--eula-already-accepted", action="store_true")
    parser.add_argument("--reload", action="store_true",help="also exercise F3+T and the compiled Mods list in a local world")
    arguments = parser.parse_args()
    for field in ["binary", "ctl", "resources"]:
        setattr(arguments, field, getattr(arguments, field).resolve(strict=True))
    dedicated(arguments) if arguments.mode == "dedicated" else local(arguments)
