#!/usr/bin/env python3
"""macOS-only PID isolation smoke test using two disposable, same-name Cocoa apps.

Build first: cd cli && cargo build -p agcu
Run from repo root: python3 scripts/smoke-pid-targeting.py
Uses existing Accessibility/Automation permissions; never changes permissions.
"""

import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
AGCU = ROOT / "cli/target/debug/agcu"


def command(*args, success=True):
    result = subprocess.run([str(AGCU), *map(str, args)], capture_output=True, text=True, timeout=20)
    if success and result.returncode:
        raise AssertionError(f"{args}: {result.stderr}")
    return result


def tree(pid):
    return json.loads(command("tree", "--pid", pid).stdout)


def nodes(node):
    yield node
    for child in node.get("children", []):
        yield from nodes(child)


def element(pid, identifier):
    return next(node for node in nodes(tree(pid)) if node.get("id") == identifier)


def eventually(check):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        try:
            if check():
                return
        except (AssertionError, StopIteration, json.JSONDecodeError):
            pass
        time.sleep(0.1)
    raise AssertionError("fixture state did not converge")


def main():
    if sys.platform != "darwin":
        raise SystemExit("This smoke test requires macOS")
    parent = Path(tempfile.gettempdir()) / "opencode"
    parent.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="agcu-pid-", dir=parent) as directory:
        os.environ["HOME"] = directory
        binary = Path(directory) / "agcu-pid-fixture"
        subprocess.run(["swiftc", str(ROOT / "cli/tests/fixtures/pid-app.swift"), "-o", str(binary)], check=True)
        apps = []
        try:
            apps = [subprocess.Popen([str(binary), title], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    for title in ("PID fixture A", "PID fixture B")]
            a, b = (app.pid for app in apps)
            eventually(lambda: element(a, "pid-button")["name"] == "Count 0" and element(b, "pid-button")["name"] == "Count 0")
            running = {app["pid"]: app["name"] for app in json.loads(command("apps").stdout)}
            assert running[a] == running[b]
            for pid in (a, b):
                assert tree(pid)["pid"] == pid
                windows = json.loads(command("windows", "--pid", pid).stdout)
                assert windows and all(window["pid"] == pid for window in windows)
            snapshot = command("snapshot", "--pid", a, "-i", "-c").stdout
            assert "Count 0" in snapshot
            ref = re.search(r"\[(@e\d+)\].*id=pid-button", snapshot).group(1)
            conflict = command("click", ref, "--pid", b, success=False)
            assert conflict.returncode == 1 and "conflicts with --pid" in conflict.stderr
            command("click", ref)
            eventually(lambda: element(a, "pid-button")["name"] == "Count 1")
            assert element(b, "pid-button")["name"] == "Count 0"
            command("click", 'id="pid-button"', "--pid", a, "--count", 2)
            eventually(lambda: element(a, "pid-button")["name"] != "Count 1")
            assert element(b, "pid-button")["name"] == "Count 0"
            button = element(a, "pid-button")
            x = button["position"]["x"] + button["size"]["width"] / 2
            y = button["position"]["y"] + button["size"]["height"] / 2
            command("click", "--x", x, "--y", y, "--pid", a)
            eventually(lambda: element(a, "pid-button")["name"] != button["name"])
            assert element(b, "pid-button")["name"] == "Count 0"
            command("click", 'id="pid-field"', "--pid", a, "--count", 2)
            command("type", "pid-isolated", "--pid", a)
            try:
                eventually(lambda: element(a, "pid-field").get("value") == "pid-isolated")
            except AssertionError:
                print("field after typing:", element(a, "pid-field"), flush=True)
                print("other field:", element(b, "pid-field"), flush=True)
                raise
            assert element(b, "pid-field").get("value") == "untouched"
            command("key", "cmd+a", "--pid", a)
            command("key", "Backspace", "--pid", a)
            command("type", "after-key", "--pid", a, "--append")
            eventually(lambda: element(a, "pid-field").get("value") == "after-key")
            assert element(b, "pid-field").get("value") == "untouched"
            for grid in (False, True):
                path = Path(directory) / f"fixture-grid-{grid}.png"
                command("screenshot", "--pid", a, "--path", path, *(["--grid"] if grid else []))
                assert path.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"
            command("snapshot", "--pid", a, "--cdp", "--cdp-port", "1", "-i", "-c")
            stale = command("windows", "--pid", 2147483647, success=False)
            assert stale.returncode == 1 and "application not found" in stale.stderr
            snapshot = command("snapshot", "--pid", b, "-i", "-c").stdout
            ref = re.search(r"\[(@e\d+)\].*id=pid-button", snapshot).group(1)
            apps[1].terminate()
            apps[1].wait(timeout=10)
            stale_ref = command("click", ref, success=False)
            assert stale_ref.returncode == 1 and "application not found" in stale_ref.stderr
            print(f"PASS: same-name PIDs {a}/{b}; discovery, refs, AXPress, coordinate fallback, typing, key effects, screenshots/grid, forced-CDP bypass, stale PID")
        finally:
            for app in apps:
                if app.poll() is None:
                    app.terminate()
                app.wait(timeout=10)


if __name__ == "__main__":
    main()
