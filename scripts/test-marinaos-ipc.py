#!/usr/bin/env python3
"""Exercise built binaries using only a private temporary socket directory."""
import json
import os
import pathlib
import subprocess
import tempfile
import time

root = pathlib.Path(__file__).resolve().parent.parent
with tempfile.TemporaryDirectory(prefix="marinaos-ipc-") as directory:
    env = os.environ.copy()
    env["MARINAOS_SOCKET"] = str(pathlib.Path(directory) / "service.sock")
    daemon = subprocess.Popen([str(root / "target/debug/marinaosd")], env=env,
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        for _ in range(100):
            if pathlib.Path(env["MARINAOS_SOCKET"]).exists():
                break
            if daemon.poll() is not None:
                raise RuntimeError("daemon exited before socket creation")
            time.sleep(0.02)
        for command in ("version", "status", "doctor", "hardware", "capabilities"):
            output = subprocess.check_output([str(root / "target/debug/marinaosctl"), command, "--json"], env=env)
            result = json.loads(output)
            assert result is not None, command
        print("all CLI commands succeeded over authenticated local IPC")
    finally:
        daemon.terminate()
        daemon.wait(timeout=5)
        assert not pathlib.Path(env["MARINAOS_SOCKET"]).exists(), "socket not removed on shutdown"
