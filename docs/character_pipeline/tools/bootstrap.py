#!/usr/bin/env python3
"""Safe Ubuntu toolchain bootstrap for the omoba character pipeline.

This script intentionally avoids modifying shared Windows/Linux AI tool roots.
It may create and delete a tiny probe file in the shared root to verify write
access. Prepare mode creates user-local Linux virtual environments only.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import venv
from pathlib import Path
from typing import Any


DEFAULT_SHARED_ROOT = Path("/media/damody/新增磁碟區/AI_Pic")
DEFAULT_LINUX_VENV_ROOT = Path("~/.cache/omoba-character-pipeline/venvs").expanduser()
DEFAULT_VENV_NAME = "character-pipeline"
DEFAULT_CACHE_ROOT = Path("~/.cache/omoba-character-pipeline").expanduser()


def run_cmd(args: list[str], timeout: int = 10) -> tuple[int, str]:
    try:
        proc = subprocess.run(
            args,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=timeout,
        )
        return proc.returncode, proc.stdout.strip()
    except FileNotFoundError:
        return 127, f"command not found: {args[0]}"
    except subprocess.TimeoutExpired:
        return 124, f"command timed out: {' '.join(args)}"


def item(name: str, status: str, kind: str, message: str, **extra: Any) -> dict[str, Any]:
    out: dict[str, Any] = {
        "name": name,
        "status": status,
        "kind": kind,
        "message": message,
    }
    out.update(extra)
    return out


def check_gpu() -> dict[str, Any]:
    code, out = run_cmd(
        [
            "nvidia-smi",
            "--query-gpu=name,driver_version,memory.total",
            "--format=csv,noheader",
        ]
    )
    if code == 0 and out:
        return item("gpu", "ok", "Info", out)
    return item("gpu", "error", "ToolchainError", out or "nvidia-smi unavailable")


def check_python() -> dict[str, Any]:
    return item("python", "ok", "Info", sys.version.split()[0], executable=sys.executable)


def venv_python(venv_path: Path) -> Path:
    return venv_path / "bin" / "python"


def check_torch(python_executable: Path | None = None) -> dict[str, Any]:
    python = str(python_executable or Path(sys.executable))
    code, out = run_cmd(
        [
            python,
            "-c",
            "import torch; print('torch', torch.__version__); print('cuda', torch.cuda.is_available())",
        ]
    )
    if code != 0:
        return item("torch_cuda", "error", "ToolchainError", out, executable=python)
    lines = out.splitlines()
    cuda = len(lines) >= 2 and lines[1].strip() == "cuda True"
    if not cuda:
        return item("torch_cuda", "error", "ToolchainError", out or "torch cuda unavailable", executable=python)
    return item("torch_cuda", "ok", "Info", out, executable=python)


def check_path(path: Path, name: str, required_files: list[str]) -> dict[str, Any]:
    if not path.exists():
        return item(name, "error", "ToolchainError", f"missing path: {path}", path=str(path))
    missing = [rel for rel in required_files if not (path / rel).exists()]
    if missing:
        return item(
            name,
            "error",
            "ToolchainError",
            f"path exists but required files are missing: {', '.join(missing)}",
            path=str(path),
        )
    return item(name, "ok", "Info", f"path available: {path}", path=str(path))


def check_shared_root(root: Path) -> list[dict[str, Any]]:
    results: list[dict[str, Any]] = []
    if not root.exists():
        return [item("shared_root", "error", "SharedRootError", f"missing shared root: {root}")]
    if not root.is_dir():
        return [item("shared_root", "error", "SharedRootError", f"not a directory: {root}")]
    results.append(item("shared_root_read", "ok", "Info", f"readable: {root}", path=str(root)))

    probe = root / f".omoba_character_pipeline_probe_{os.getpid()}.tmp"
    try:
        probe.write_text("probe\n", encoding="utf-8")
        probe.unlink()
        results.append(item("shared_root_write_probe", "ok", "Info", "probe file created and deleted"))
    except OSError as exc:
        results.append(item("shared_root_write_probe", "error", "SharedRootError", str(exc)))
        try:
            if probe.exists():
                probe.unlink()
        except OSError:
            pass
    return results


def check_windows_markers(shared_root: Path) -> list[dict[str, Any]]:
    markers = [
        shared_root / "stable-diffusion-webui" / "venv" / "Scripts" / "python.exe",
        shared_root / "ComfyUI" / "python_embeded" / "python.exe",
    ]
    results: list[dict[str, Any]] = []
    for marker in markers:
        if marker.exists():
            results.append(
                item(
                    "windows_runtime_marker",
                    "warn",
                    "Info",
                    "Windows runtime marker detected; bootstrap must not modify it",
                    path=str(marker),
                )
            )
    return results


def check_executable(
    command: str,
    name: str,
    args: list[str] | None = None,
    fallback: Path | None = None,
) -> dict[str, Any]:
    exe = shutil.which(command)
    if not exe and fallback and fallback.exists():
        exe = str(fallback)
    if not exe:
        return item(name, "error", "ToolchainError", f"{command} not found on PATH")
    code, out = run_cmd([exe] + (args or []), timeout=10)
    if code != 0:
        return item(name, "error", "ToolchainError", out or f"{command} failed", executable=exe)
    return item(name, "ok", "Info", out.splitlines()[0] if out else exe, executable=exe)


def check_venv(venv_path: Path) -> dict[str, Any]:
    python = venv_python(venv_path)
    if not python.exists():
        return item("linux_venv", "error", "ToolchainError", f"venv python missing: {python}", path=str(venv_path))
    code, out = run_cmd([str(python), "--version"])
    if code != 0:
        return item("linux_venv", "error", "ToolchainError", out, path=str(venv_path))
    return item("linux_venv", "ok", "Info", out, path=str(venv_path), executable=str(python))


def prepare_venv(venv_path: Path) -> dict[str, Any]:
    if venv_path.exists() and venv_python(venv_path).exists():
        return item("prepare_venv", "ok", "Info", "venv already exists", path=str(venv_path))
    venv_path.parent.mkdir(parents=True, exist_ok=True)
    builder = venv.EnvBuilder(with_pip=True, clear=False, symlinks=True)
    builder.create(venv_path)
    return item("prepare_venv", "ok", "Info", "created Linux venv", path=str(venv_path))


def prepare(shared_root: Path, linux_venv_root: Path, venv_name: str) -> dict[str, Any]:
    venv_path = linux_venv_root / venv_name
    checks: list[dict[str, Any]] = []
    checks.extend(check_shared_root(shared_root))
    checks.extend(check_windows_markers(shared_root))
    checks.append(prepare_venv(venv_path))
    checks.append(check_venv(venv_path))
    return {
        "schema_version": 1,
        "mode": "prepare",
        "shared_root": str(shared_root),
        "linux_venv_root": str(linux_venv_root),
        "venv_path": str(venv_path),
        "venv_python": str(venv_python(venv_path)),
        "checks": checks,
        "summary": {
            "errors": sum(1 for check in checks if check["status"] == "error"),
            "warnings": sum(1 for check in checks if check["status"] == "warn"),
        },
    }


def find_blender(cache_root: Path) -> Path | None:
    exe = shutil.which("blender")
    if exe:
        return Path(exe)
    candidates = sorted(cache_root.glob("blender*/blender")) + sorted(cache_root.glob("blender/blender*/blender"))
    return candidates[0] if candidates else None


def diagnose(shared_root: Path, linux_venv_root: Path, venv_name: str, cache_root: Path) -> dict[str, Any]:
    comfy_root = shared_root / "ComfyUI" / "ComfyUI"
    webui_root = shared_root / "stable-diffusion-webui"
    venv_path = linux_venv_root / venv_name
    python = venv_python(venv_path) if venv_python(venv_path).exists() else None
    blender = find_blender(cache_root)

    checks: list[dict[str, Any]] = [
        check_gpu(),
        check_python(),
        check_venv(venv_path),
        check_torch(python),
        item(
            "linux_venv_root",
            "ok" if linux_venv_root.parent.exists() else "warn",
            "Info",
            "Linux venvs should be created outside the shared AI root",
            path=str(linux_venv_root),
        ),
    ]
    checks.extend(check_shared_root(shared_root))
    checks.append(check_path(comfy_root, "comfyui_root", ["main.py", "models"]))
    checks.append(check_path(webui_root, "stable_diffusion_webui_root", ["launch.py", "models"]))
    checks.extend(check_windows_markers(shared_root))
    checks.append(check_executable("blender", "blender", ["--version"], fallback=blender))
    checks.append(
        check_executable(
            "lua5.4",
            "lua5.4",
            ["-v"],
            fallback=Path("~/.cache/omoba-character-pipeline/bin/lua5.4").expanduser(),
        )
    )

    return {
        "schema_version": 1,
        "mode": "diagnose",
        "shared_root": str(shared_root),
        "linux_venv_root": str(linux_venv_root),
        "venv_path": str(venv_path),
        "venv_python": str(venv_python(venv_path)),
        "cache_root": str(cache_root),
        "checks": checks,
        "summary": {
            "errors": sum(1 for check in checks if check["status"] == "error"),
            "warnings": sum(1 for check in checks if check["status"] == "warn"),
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Prepare or diagnose omoba character pipeline toolchain")
    parser.add_argument("--diagnose", action="store_true", help="run safe diagnostics")
    parser.add_argument("--prepare", action="store_true", help="create user-local Linux venv")
    parser.add_argument("--shared-root", default=str(DEFAULT_SHARED_ROOT))
    parser.add_argument("--linux-venv-root", default=str(DEFAULT_LINUX_VENV_ROOT))
    parser.add_argument("--venv-name", default=DEFAULT_VENV_NAME)
    parser.add_argument("--cache-root", default=str(DEFAULT_CACHE_ROOT))
    args = parser.parse_args()

    if args.prepare == args.diagnose:
        parser.error("choose exactly one of --prepare or --diagnose")

    shared_root = Path(args.shared_root)
    linux_venv_root = Path(args.linux_venv_root).expanduser()
    cache_root = Path(args.cache_root).expanduser()
    if args.prepare:
        report = prepare(shared_root, linux_venv_root, args.venv_name)
    else:
        report = diagnose(shared_root, linux_venv_root, args.venv_name, cache_root)
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
