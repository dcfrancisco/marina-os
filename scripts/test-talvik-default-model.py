#!/usr/bin/env python3
"""Bounded default-model acceptance using an existing local GGUF and isolated state.

Build talvik/talvikd first. No downloads or installed state/model writes. The
synthetic large-model entry is invalid and must never be chosen as fallback.
"""
import argparse
import contextlib
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request


def request(base, path, body=None):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(base + path, data=data,
                                 headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            return response.status, json.load(response)
    except urllib.error.HTTPError as error:
        return error.code, json.load(error)


@contextlib.contextmanager
def daemon(binary, root, override=None):
    env = {k: v for k, v in os.environ.items()
           if not k.startswith(("TALVIK_", "MARINA_", "OID_"))}
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
    env.update(TALVIK_HOME=str(root), TALVIK_CONFIG=str(root / "config.yaml"),
               TALVIK_STATE_DIR=str(root), TALVIK_MODEL_DIR=str(root / "models"),
               TALVIK_REGISTRY=str(root / "models.registry"),
               TALVIK_SOCKET=str(root / "talvik.sock"),
               TALVIK_HTTP_ADDR=f"127.0.0.1:{port}", TALVIK_HTTP_AUTH="false",
               TALVIK_THREADS="2", TALVIK_PLACEMENT="cpu_only", TALVIK_BACKEND="cpu")
    if override is not None:
        env["TALVIK_DEFAULT_MODEL"] = override
    base = f"http://127.0.0.1:{port}"
    with tempfile.TemporaryFile(mode="w+") as log:
        process = subprocess.Popen([str(binary)], env=env, stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 45
            while time.monotonic() < deadline:
                assert process.poll() is None, "daemon exited"
                try:
                    code, ready = request(base, "/readyz")
                    if code == 200 or ready.get("bootstrap") not in ("starting", "provisioning"):
                        break
                except (OSError, ValueError):
                    pass
                time.sleep(0.1)
            else:
                raise AssertionError("bootstrap timed out")
            try:
                yield base, env, log
            except BaseException:
                log.seek(0)
                print(log.read()[-10000:])
                raise
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
            if process.returncode not in (0, -15):
                log.seek(0)
                print(log.read())


def models(base):
    code, data = request(base, "/v1/models")
    assert code == 200, data
    return data["data"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", type=Path, required=True, help="existing small local GGUF")
    parser.add_argument("--bin-dir", type=Path, default=Path("target/debug"))
    args = parser.parse_args()
    model = args.model.resolve(strict=True)
    binary = (args.bin_dir / "talvikd").resolve(strict=True)
    client = (args.bin_dir / "talvik").resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="talvik-default-") as directory:
        root = Path(directory)
        (root / "models").mkdir()
        for name in ("small", "explicit"):
            (root / "models" / f"{name}.gguf").symlink_to(model)
        (root / "models/aaa-large-model.gguf").write_bytes(b"GGUF invalid fixture")
        config = root / "config.yaml"
        config.write_text("default_model: small\nauto_download: false\nauto_load: true\n")
        # Reuse persisted registry over a full process restart; ordering cannot win.
        for _ in range(2):
            with daemon(binary, root) as (base, env, _log):
                entries = models(base)
                assert [m["id"] for m in entries if m["default"]] == ["small"], entries
                assert [m["id"] for m in entries if m["resident"]] == ["small"], entries
                code, result = request(base, "/v1/completions", {
                    "model": "small", "prompt": "Hello", "max_tokens": 2, "context_size": 512})
                assert code == 200 and result["model"] == "small", result
                if _ == 0:
                    default_run = subprocess.run([str(client), "--plain", "run", "--default", "Reply only OK."],
                                                 env=env, capture_output=True, text=True, timeout=30)
                    assert default_run.returncode == 0 and default_run.stdout.strip(), default_run
                # Explicit CLI selection must be sent verbatim, never replaced by default.
                result = subprocess.run([str(client), "--plain", "run", "missing-explicit", "Hi"],
                                        env=env, capture_output=True, text=True, timeout=15)
                assert result.returncode != 0 and "missing-explicit" in result.stderr, result
        # Environment overrides YAML, including the existing opt-out.
        for override, expected in (("explicit", ["explicit"]), ("none", [])):
            with daemon(binary, root, override) as (base, _env, _log):
                entries = models(base)
                assert [m["id"] for m in entries if m["default"]] == expected, entries
                assert [m["id"] for m in entries if m["resident"]] == expected, entries
        for selected, diagnostic in (("missing", "not installed"),
                                     ("aaa-large-model", "was not loaded")):
            config.write_text(f"default_model: {selected}\nauto_download: false\nauto_load: true\n")
            with daemon(binary, root) as (base, env, log):
                code, _ = request(base, "/readyz")
                assert code == 503
                assert not any(m["resident"] for m in models(base))
                log.seek(0)
                assert diagnostic in log.read()
                # --default reports this configured failure instead of falling back.
                result = subprocess.run([str(client), "--plain", "run", "--default", "Hi"],
                                        env=env, capture_output=True, text=True, timeout=15)
                assert result.returncode != 0 and selected in result.stderr, result
                assert not any(m["resident"] for m in models(base))
    print("PASS: YAML default, restart, inference, explicit CLI/API selection, environment override, opt-out, missing/incompatible diagnostics; no large-model fallback")


if __name__ == "__main__":
    main()
