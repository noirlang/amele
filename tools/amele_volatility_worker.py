#!/usr/bin/env python3
"""Amele Volatility3 worker.

This small wrapper keeps Volatility3's Python-specific setup in Python while
the Rust application stays responsible for UI, jobs, and JSON parsing.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


LINUX_BANNER_NEEDLE = b"Linux version "
LINUX_BANNER_MAX_LEN = 320
LINUX_BANNER_SCAN_CHUNK = 8 * 1024 * 1024


def log(message: str) -> None:
    print(f"[amele-volatility-worker] {message}", file=sys.stderr, flush=True)


def resolve_vol_invocation(explicit: str | None) -> tuple[list[str], Path | None, str, dict[str, str]]:
    candidates: list[Path] = []
    if explicit:
        candidates.append(Path(explicit))
    for key in ("AMELE_VOLATILITY3_PATH", "VOLATILITY3_PATH"):
        value = os.environ.get(key)
        if value:
            candidates.append(Path(value))

    cwd = Path.cwd()
    script_dir = Path(__file__).resolve().parent
    candidates.extend(
        [
            cwd / "volatility3",
            cwd / "vendor/volatility3",
            cwd / "../volatility3",
            cwd / "../vendor/volatility3",
            cwd / "../../volatility3",
            cwd / "../../vendor/volatility3",
            script_dir / "../volatility3",
            script_dir / "../vendor/volatility3",
            script_dir / "../../volatility3",
            script_dir / "../../vendor/volatility3",
        ]
    )

    for candidate in candidates:
        vol_py = candidate if candidate.name == "vol.py" else candidate / "vol.py"
        if vol_py.exists():
            root = vol_py.resolve().parent
            env = os.environ.copy()
            env["PYTHONPATH"] = f"{root}{os.pathsep}{env.get('PYTHONPATH', '')}"
            return [sys.executable, str(vol_py.resolve())], root, str(vol_py.resolve()), env

    vol_cmd = shutil.which("vol")
    if vol_cmd:
        return [vol_cmd], None, vol_cmd, os.environ.copy()

    raise SystemExit("Volatility3 vol.py not found")


def symbol_dirs_arg(symbol_dirs: list[str]) -> str | None:
    clean = [str(Path(item).resolve()) for item in symbol_dirs if item and Path(item).exists()]
    return ";".join(dict.fromkeys(clean)) if clean else None


def cache_dir() -> Path:
    roots: list[Path] = []
    if os.environ.get("XDG_CACHE_HOME"):
        roots.append(Path(os.environ["XDG_CACHE_HOME"]))
    if os.environ.get("HOME"):
        roots.append(Path(os.environ["HOME"]) / ".cache")
    roots.append(Path(tempfile.gettempdir()))

    for root in roots:
        path = root / "amele" / "volatility3"
        try:
            path.mkdir(parents=True, exist_ok=True)
            return path
        except OSError:
            continue
    return Path(tempfile.gettempdir())


def run_volatility(args: argparse.Namespace, plugin_args: list[str]) -> int:
    cmd, cwd, display, env = resolve_vol_invocation(args.vol_py)
    symbols = symbol_dirs_arg(args.symbol_dir or [])
    if symbols:
        cmd.extend(["-s", symbols])
    cmd.extend(["--cache-path", str(cache_dir())])
    cmd.extend(["-q", "-f", args.file, "-r", "json"])
    cmd.extend(plugin_args)

    log(f"volatility: {display}")
    log(f"image: {args.file}")
    if symbols:
        log(f"symbol-dirs: {symbols}")
    log(f"plugin: {' '.join(plugin_args)}")

    proc = subprocess.run(cmd, text=True, capture_output=True, check=False, cwd=cwd, env=env)
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")
    if proc.stdout:
        print(proc.stdout, end="")
    return proc.returncode


def scan_linux_banners(path: Path, limit: int) -> list[str]:
    found: dict[str, None] = {}
    carry = b""
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(LINUX_BANNER_SCAN_CHUNK)
            if not chunk:
                break
            data = carry + chunk
            cursor = 0
            while True:
                pos = data.find(LINUX_BANNER_NEEDLE, cursor)
                if pos == -1:
                    break
                banner = extract_banner(data[pos : pos + LINUX_BANNER_MAX_LEN])
                if banner and looks_like_kernel_banner(banner):
                    found.setdefault(banner, None)
                    if len(found) >= limit:
                        return list(found)
                cursor = pos + len(LINUX_BANNER_NEEDLE)
            keep = LINUX_BANNER_MAX_LEN + len(LINUX_BANNER_NEEDLE)
            carry = data[-keep:]
    return list(found)


def extract_banner(raw: bytes) -> str | None:
    end = 0
    for idx, byte in enumerate(raw[:LINUX_BANNER_MAX_LEN]):
        if byte in (0, 10, 13):
            break
        if byte < 32 or byte > 126:
            break
        end = idx + 1
    text = raw[:end].decode("utf-8", errors="ignore").strip()
    return text or None


def looks_like_kernel_banner(text: str) -> bool:
    return (
        text.startswith("Linux version ")
        and "%s" not in text
        and "http" not in text
        and any(marker in text for marker in (" SMP ", "PREEMPT", "GNU ld", "gcc", "#1"))
    )


def preflight(args: argparse.Namespace) -> int:
    image = Path(args.file)
    result: dict[str, object] = {
        "banners": [],
        "warnings": [],
        "recommendations": [],
    }
    try:
        _, _, display, _ = resolve_vol_invocation(args.vol_py)
        result["vol_py"] = display
    except SystemExit as exc:
        result["vol_py"] = None
        result["warnings"].append(str(exc))

    if args.os_type == "linux":
        banners = scan_linux_banners(image, args.banner_limit)
        result["banners"] = banners
        if not banners:
            result["warnings"].append("No Linux kernel banner candidate found")
        else:
            result["recommendations"].append(
                "Create or select a Volatility3 Linux ISF symbol file matching the first banner exactly"
            )

    print(json.dumps(result, ensure_ascii=False))
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Amele Volatility3 worker")
    sub = parser.add_subparsers(dest="command", required=True)

    plugin = sub.add_parser("plugin")
    plugin.add_argument("--vol-py")
    plugin.add_argument("--file", required=True)
    plugin.add_argument("--symbol-dir", action="append", default=[])
    plugin.add_argument("plugin_args", nargs=argparse.REMAINDER)

    pf = sub.add_parser("preflight")
    pf.add_argument("--vol-py")
    pf.add_argument("--file", required=True)
    pf.add_argument("--os-type", choices=("windows", "linux"), default="windows")
    pf.add_argument("--symbol-dir", action="append", default=[])
    pf.add_argument("--banner-limit", type=int, default=1)

    args = parser.parse_args(argv)
    if args.command == "plugin":
        plugin_args = list(args.plugin_args)
        if plugin_args and plugin_args[0] == "--":
            plugin_args = plugin_args[1:]
        if not plugin_args:
            raise SystemExit("Plugin name is required")
        return run_volatility(args, plugin_args)
    if args.command == "preflight":
        return preflight(args)
    raise SystemExit(f"Unknown command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
